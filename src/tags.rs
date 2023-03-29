use crate::{
    models::{AppState, Game, Tag},
    security::RequireApiKey,
};
use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use sqlx::{query, query_as};

#[utoipa::path(
    context_path = "/tags",
    responses(
        (status = 200, description = "List all tags", body = [Tag]),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/")]
pub async fn get_all_tags(state: Data<AppState>) -> impl Responder {
    match query_as::<_, Tag>("SELECT * FROM tags ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
    {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/tags",
    request_body(content=Tag, content_type="application/json", description="Tag Information"),
    responses(
        (status = 201, description = "Created new tag"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[post("/", wrap = "RequireApiKey")]
pub async fn add_tag(state: Data<AppState>, tag: Json<Tag>) -> impl Responder {
    match query("INSERT INTO tags VALUES ($1, $2)")
        .bind(&tag.name)
        .bind(&tag.description)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Created().json(Tag {
            name: tag.name.clone(),
            description: tag.description.clone(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/tags",
    responses(
        (status = 200, description = "Get specified tag", body = Tag),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/{tag}")]
pub async fn get_tag(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (name,) = path.into_inner();
    match query_as::<_, Tag>("SELECT * FROM tags WHERE name = $1")
        .bind(name)
        .fetch_one(&state.db)
        .await
    {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(_) => HttpResponse::BadRequest().body("Tag Does Not Exist"),
    }
}

#[utoipa::path(
    context_path = "/tags",
    responses(
        (status = 200, description = "Delete tag"),
        (status = 400, description = "Missing tag"),
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
#[delete("/{tag}", wrap = "RequireApiKey")]
pub async fn delete_tag(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (name,) = path.into_inner();
    if query_as::<_, Tag>("SELECT * FROM tags WHERE name = $1")
        .bind(&name)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Tag Does Not Exist");
    }
    match query("DELETE FROM tags WHERE name = $1; DELETE FROM game_tags WHERE tag_name = $1")
        .bind(name)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/tags",
    request_body(content=Tag, content_type="application/json", description="Tag Information"),
    responses(
        (status = 201, description = "Updated tag"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[put("/{tag}", wrap = "RequireApiKey")]
pub async fn edit_tag(
    state: Data<AppState>,
    path: Path<(String,)>,
    tag: Json<Tag>,
) -> impl Responder {
    let (name,) = path.into_inner();
    if query_as::<_, Tag>("SELECT * FROM tags WHERE name = $1")
        .bind(&name)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Tag Does Not Exist");
    }
    match query("UPDATE tags SET name = $1, description = $2 WHERE name = $3; UPDATE game_tags SET tag_name = $1 WHERE tag_name = $3")
        .bind(&tag.name)
        .bind(&tag.description)
        .bind(name)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Created().json(Tag {
            name: tag.name.clone(),
            description: tag.description.clone(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/tags",
    responses(
        (status = 200, description = "Get games with tag", body = [Game]),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/{tag}/games")]
pub async fn get_tag_games(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (name,) = path.into_inner();
    if query_as::<_, Tag>("SELECT * FROM tags WHERE name = $1")
        .bind(&name)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Tag Does Not Exist");
    }
    match query_as::<_, Game>("SELECT game.* FROM game LEFT JOIN game_tags ON game_tags.game_id = game.id LEFT JOIN tags ON tags.name = game_tags.tag_name WHERE game_tags.tag_name = $1 GROUP BY game.id ORDER BY name ASC")
        .bind(name)
        .fetch_all(&state.db)
        .await
    {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
