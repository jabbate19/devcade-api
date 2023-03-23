use crate::{
    models::{AppState, Game},
    security::RequireApiKey,
};
use actix_multipart::form::{
    text::Text,
    tempfile::TempFile, MultipartForm
};
use actix_web::{
    delete, get, HttpResponse, Responder,post, put,
    web::{Data, Path},
};
use aws_sdk_s3::{Client, types::ByteStream};
use chrono::prelude::*;
use data_encoding::HEXLOWER;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{query, query_as};
use std::{
    env,
    error::Error,
    fmt,
    fs::{remove_dir_all, remove_file, File},
    process::Command,
};
use utoipa::ToSchema;
use uuid::Uuid;
use zip::read::ZipArchive;

lazy_static! {
    static ref GAMES_BUCKET: String = env::var("S3_GAMES_BUCKET").unwrap();
}

#[derive(Debug, Clone)]

struct GameError {
    reason: String,
}

impl GameError {
    fn new(reason: &str) -> GameError {
        GameError {
            reason: reason.to_string(),
        }
    }
}
impl Error for GameError {}
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GameData {
    #[schema(example = "BrickBreaker")]
    name: String,
    #[schema(example = "Break bricks, get points")]
    description: String,
    #[schema(example = "ella")]
    author: String,
    #[schema(example = false)]
    authrequired: bool,
}

#[derive(Debug, MultipartForm)]
pub struct GameUpload {
    file: TempFile,
    title: Text<String>,
    description: Text<String>,
    author: Text<String>,
}

#[allow(dead_code)]
#[derive(ToSchema)]
pub struct GameUploadDoc {
    #[schema(format = Binary)]
    file: String,
    title: String,
    description: String,
    author: String,
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "List all games", body = [Game]),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/")]
pub async fn get_all_games(state: Data<AppState>) -> impl Responder {
    match query_as::<_, Game>("SELECT * FROM game")
        .fetch_all(&state.db)
        .await
    {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn verify_and_upload(
    zip: TempFile,
    s3: &Client,
    uuid: Option<String>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let content_type = zip
        .content_type
        .as_ref()
        .ok_or("Could not determine file type")?
        .clone();
    if content_type != "application/zip" {
        return Err(Box::new(GameError::new("File provided is not a Zip")));
    }
    let uuid = uuid.unwrap_or(Uuid::new_v4().to_string());
    let zip_file = File::open(zip.file.path())?;
    let mut zip_archive = ZipArchive::new(zip_file)?;
    {
        let publish = zip_archive
            .by_name("publish/")
            .map_err(|_| GameError::new("publish directory not found"))?;
        if !publish.is_dir() {
            return Err(Box::new(GameError::new("publish is not a zip")));
        }
    }
    {
        let banner = zip_archive
            .by_name("banner.png")
            .map_err(|_| GameError::new("banner not found"))?;
        if !banner.is_file() {
            return Err(Box::new(GameError::new("banner.png is not a file")));
        }
    }
    {
        let icon = zip_archive
            .by_name("icon.png")
            .map_err(|_| GameError::new("icon not found"))?;
        if !icon.is_file() {
            return Err(Box::new(GameError::new("icon.png is not a file")));
        }
    }
    let _ = zip_archive.extract(format!("/tmp/{}", uuid))?;
    let _ = s3
        .put_object()
        .key(format!("{}/banner.png", uuid))
        .body(ByteStream::from_path(format!("/tmp/{}/banner.png", uuid)).await?)
        .bucket(&GAMES_BUCKET.to_string())
        .send()
        .await?;
    let _ = s3
        .put_object()
        .key(format!("{}/icon.png", uuid))
        .body(ByteStream::from_path(format!("/tmp/{}/icon.png", uuid)).await?)
        .bucket(&GAMES_BUCKET.to_string())
        .send()
        .await?;
    let _zip_cmd = Command::new("/usr/bin/zip")
        .arg("-r")
        .arg(format!("/tmp/{}.zip", uuid))
        .arg(format!("/tmp/{}/publish", uuid))
        .spawn()?.wait()?;
    let hash = {
        let mut publish_zip = File::open(format!("/tmp/{}.zip", uuid)).unwrap();
        let mut hasher = Sha256::new();
        let _bytes_written = std::io::copy(&mut publish_zip, &mut hasher);
        let result = hasher.finalize();
        HEXLOWER.encode(&result)
    };
    let _ = s3
        .put_object()
        .key(format!("{}/{}.zip", uuid, uuid))
        .body(ByteStream::from_path(format!("/tmp/{}.zip", uuid)).await?)
        .bucket(&GAMES_BUCKET.to_string())
        .send()
        .await?;
    remove_dir_all(format!("/tmp/{}", uuid))?;
    remove_file(format!("/tmp/{}.zip", uuid))?;
    Ok((uuid.to_string(), hash))
}

#[utoipa::path(
    context_path = "/games",
    request_body(content=GameUploadDoc, content_type="multipart/form-data", description="Multipart Form. Contains zip file of game data (banner.png, icon.png, and public folder) and JSON with name, desc, and author"),
    responses(
        (status = 201, description = "Created new game"),
        (status = 400, description = "Invalid format of file upload"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[post("/", wrap = "RequireApiKey")]
pub async fn add_game(
    state: Data<AppState>,
    MultipartForm(form): MultipartForm<GameUpload>,
) -> impl Responder {
    match verify_and_upload(form.file, &state.s3, None).await {
        Ok((uuid, hash)) => {
            let date = Local::now().date_naive();
            match query("INSERT INTO game VALUES ($1, $2, $3, $4, $5, $6, $7)")
                .bind(&uuid)
                .bind(form.author.clone())
                .bind(date)
                .bind(form.title.clone())
                .bind(&hash)
                .bind(form.description.clone())
                .bind(false)
                .execute(&state.db)
                .await
            {
                Ok(_) => HttpResponse::Created().json(Game {
                    game_id: uuid,
                    author_username: form.author.clone(),
                    upload_date: date,
                    game_name: form.title.clone(),
                    hash: hash,
                    description: form.description.clone(),
                    authrequired: false,
                }),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(e) => HttpResponse::NotAcceptable().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "List all games", body = [Game]),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/{id}")]
pub async fn get_game(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (id,) = path.into_inner();
    match query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
    {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(_) => HttpResponse::BadRequest().body("Game ID Does Not Exist"),
    }
}

#[utoipa::path(
    context_path = "/games",
    request_body(content=GameUploadDoc, content_type="multipart/form-data", description="Multipart Form. Contains zip file of game data (banner.png, icon.png, and public folder) and JSON with name, desc, and author"),
    responses(
        (status = 200, description = "Updated game"),
        (status = 400, description = "Invalid format of file upload"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[put("/{id}", wrap = "RequireApiKey")]
pub async fn edit_game(
    state: Data<AppState>,
    path: Path<(String,)>,
    MultipartForm(form): MultipartForm<GameUpload>,
) -> impl Responder {
    let (id,) = path.into_inner();
    match query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
    {
        Ok(game) => match verify_and_upload(form.file, &state.s3, Some(id.clone())).await {
            Ok((_, hash)) => {
                match query(
                    "UPDATE game SET game_name = $1, description = $2, hash=$3, authrequired=$4 WHERE game_id = $5",
                )
                .bind(form.title.clone())
                .bind(form.description.clone())
                .bind(&hash)
                .bind(game.authrequired)
                .bind(&id)
                .execute(&state.db)
                .await
                {
                    Ok(_) => HttpResponse::Ok().json(Game {
                        game_id: id,
                        author_username: game.author_username,
                        upload_date: game.upload_date,
                        game_name: form.title.clone(),
                        hash: hash,
                        description: form.description.clone(),
                        authrequired: game.authrequired,
                    }),
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            }
            Err(e) => HttpResponse::NotAcceptable().body(e.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().body("Game ID Does Not Exist"),
    }
}

async fn delete_recursively(s3: &Client, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    s3.delete_object().bucket(&GAMES_BUCKET.to_string()).key(format!("{}/{}.zip", id, id)).send().await?;
    s3.delete_object().bucket(&GAMES_BUCKET.to_string()).key(format!("{}/icon.png", id)).send().await?;
    s3.delete_object().bucket(&GAMES_BUCKET.to_string()).key(format!("{}/banner.png", id)).send().await?;
    Ok(())
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "Delete game"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Deletion"),
    ),
    params(
        ("id", description = "Unique id of game")
    ),
    security(
        ("api_key" = [])
    )
)]
#[delete("/{id}", wrap = "RequireApiKey")]
pub async fn delete_game(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (id,) = path.into_inner();
    if query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Game ID Does Not Exist");
    }
    match delete_recursively(&state.s3, &id).await {
        Ok(_) => {
            match query("DELETE FROM game WHERE game_id = $1")
                .bind(id)
                .execute(&state.db)
                .await
            {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "Provide game source zip", content_type="application/zip"),
        (status = 500, description = "Error Created by Query"),
    ),
    params(
        ("id", description = "Unique id of game")
    ),
)]
#[get("/{id}/game")]
pub async fn get_binary(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (id,) = path.into_inner();
    if query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Game ID Does Not Exist");
    }
    match state
        .s3
        .get_object()
        .bucket(&GAMES_BUCKET.to_string())
        .key(format!("{}/{}.zip", id, id))
        .send()
        .await
    {
        Ok(objout) => {
            let bytestream = objout.body.collect().await;
            match bytestream {
                Ok(bytes) => HttpResponse::Ok().body(bytes.into_bytes()),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Error getting object body: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error getting object: {}", e)),
    }
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "Provide game banner", content_type="application/png"),
        (status = 500, description = "Error Created by Query"),
    ),
    params(
        ("id", description = "Unique id of game")
    ),
)]
#[get("/{id}/banner")]
pub async fn get_banner(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (id,) = path.into_inner();
    if query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Game ID Does Not Exist");
    }
    match state
        .s3
        .get_object()
        .bucket(&GAMES_BUCKET.to_string())
        .key(format!("{}/banner.png", id))
        .send()
        .await
    {
        Ok(objout) => {
            let bytestream = objout.body.collect().await;
            match bytestream {
                Ok(bytes) => HttpResponse::Ok().body(bytes.into_bytes()),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Error getting object body: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error getting object: {}", e)),
    }
}

#[utoipa::path(
    context_path = "/games",
    responses(
        (status = 200, description = "Provide game icon", content_type="application/png"),
        (status = 500, description = "Error Created by Query"),
    ),
    params(
        ("id", description = "Unique id of game")
    ),
)]
#[get("/{id}/icon")]
pub async fn get_icon(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (id,) = path.into_inner();
    if query_as::<_, Game>("SELECT * FROM game WHERE game_id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Game ID Does Not Exist");
    }
    match state
        .s3
        .get_object()
        .bucket(&GAMES_BUCKET.to_string())
        .key(format!("{}/icon.png", id))
        .send()
        .await
    {
        Ok(objout) => {
            let bytestream = objout.body.collect().await;
            match bytestream {
                Ok(bytes) => HttpResponse::Ok().body(bytes.into_bytes()),
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Error getting object body: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error getting object: {}", e)),
    }
}
