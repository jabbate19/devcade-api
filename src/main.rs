use actix_cors::Cors;
use actix_web::{
    http,
    middleware::Logger,
    web::{scope, Data},
    App, HttpServer,
};
use aws_sdk_s3 as s3;
use aws_sdk_s3::Endpoint;
use devcade_api_rs::{
    games::routes::{self as games, FileUploadDoc, GameData, GameUploadDoc},
    models::{AppState, Game, GameWithTags, Tag, User, UserType},
    tags::routes as tags,
    users::routes as users,
    app::{self, configure_app, get_app_data},
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let app_data = get_app_data().await;
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&env::var("DOMAIN").unwrap())
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T",
            ))
            .configure(configure_app)
            .app_data(app_data.clone())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
