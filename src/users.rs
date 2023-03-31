use crate::{
    models::{AppState, User},
    security::RequireApiKey,
};
use actix_web::{
    get, post, put,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use sqlx::{query, query_as};

#[utoipa::path(
    context_path = "/users",
    request_body(content=User, content_type="application/json", description="User Information"),
    responses(
        (status = 201, description = "Created new user"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[post("/", wrap = "RequireApiKey")]
pub async fn add_user(state: Data<AppState>, user: Json<User>) -> impl Responder {
    match query("INSERT INTO users VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(&user.id)
        .bind(&user.user_type)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.picture)
        .bind(&user.admin)
        .bind(&user.email)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Created().json(User {
            id: user.id.clone(),
            user_type: user.user_type.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            picture: user.picture.clone(),
            admin: user.admin,
            email: user.email.clone(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    context_path = "/users",
    responses(
        (status = 200, description = "Get specified user", body = User),
        (status = 500, description = "Error Created by Query"),
    )
)]
#[get("/{uid}")]
pub async fn get_user(state: Data<AppState>, path: Path<(String,)>) -> impl Responder {
    let (uid,) = path.into_inner();
    match query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&state.db)
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::BadRequest().body("User Does Not Exist"),
    }
}

#[utoipa::path(
    context_path = "/users",
    request_body(content=User, content_type="application/json", description="User Information"),
    responses(
        (status = 201, description = "Updated user"),
        (status = 401, description = "Invalid/Missing API Key"),
        (status = 500, description = "Error Created by Query"),
    ),
    security(
        ("api_key" = [])
    )
)]
#[put("/{uid}", wrap = "RequireApiKey")]
pub async fn edit_user(
    state: Data<AppState>,
    path: Path<(String,)>,
    user: Json<User>,
) -> impl Responder {
    let (uid,) = path.into_inner();
    if query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&uid)
        .fetch_one(&state.db)
        .await
        .is_err()
    {
        return HttpResponse::BadRequest().body("Tag Does Not Exist");
    }
    match query("UPDATE users SET first_name = $1, last_name = $2, picture = $3, admin = $4, email = $5 WHERE id = $6")
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.picture)
        .bind(&user.admin)
        .bind(&user.email)
        .bind(uid)
        .execute(&state.db)
        .await
    {
        Ok(_) => HttpResponse::Created().json(User {
            id: user.id.clone(),
            user_type: user.user_type.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            picture: user.picture.clone(),
            admin: user.admin,
            email: user.email.clone(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
