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

    #[derive(OpenApi)]
    #[openapi(
        paths(
            games::get_all_games,
            games::get_game,
            games::edit_game,
            games::delete_game,
            games::add_game,
            games::get_binary,
            games::update_binary,
            games::get_banner,
            games::update_banner,
            games::get_icon,
            games::update_icon,
            tags::get_all_tags,
            tags::get_tag,
            tags::edit_tag,
            tags::delete_tag,
            tags::add_tag,
            tags::get_tag_games,
            users::get_user,
            users::add_user,
            users::edit_user,
        ),
        components(
            schemas(GameData, Game, GameUploadDoc, FileUploadDoc, GameWithTags, Tag, User, UserType)
        ),
        tags(
            (name = "DevcadeAPI", description = "")
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("frontend_api_key"))),
            )
        }
    }

    let openapi = ApiDoc::openapi();
    let shared_config = aws_config::load_from_env().await;

    // Create an S3 config from the shared config and override the endpoint resolver.
    let s3_config = s3::config::Builder::from(&shared_config)
        .endpoint_resolver(Endpoint::immutable(
            env::var("S3_ENDPOINT")
                .unwrap_or("https://s3.csh.rit.edu".to_string())
                .parse()
                .unwrap(),
        ))
        //.endpoint_resolver(s3::Endpoint::immutable("https://s3.csh.rit.edu".parse().unwrap()))
        .build();
    let s3_conn = s3::Client::from_conf(s3_config);

    let pool = PgPoolOptions::new()
        .connect(&env::var("SQL_URI").unwrap())
        .await
        .unwrap();
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
            .app_data(Data::new(AppState {
                db: pool.clone(),
                s3: s3_conn.clone(),
            }))
            .service(
                scope("/games")
                    .service(games::get_all_games)
                    .service(games::get_game)
                    .service(games::edit_game)
                    .service(games::delete_game)
                    .service(games::add_game)
                    .service(games::get_binary)
                    .service(games::update_binary)
                    .service(games::get_banner)
                    .service(games::update_banner)
                    .service(games::get_icon)
                    .service(games::update_icon),
            )
            .service(
                scope("/tags")
                    .service(tags::get_all_tags)
                    .service(tags::get_tag)
                    .service(tags::edit_tag)
                    .service(tags::delete_tag)
                    .service(tags::add_tag)
                    .service(tags::get_tag_games),
            )
            .service(
                scope("/users")
                    .service(users::get_user)
                    .service(users::add_user)
                    .service(users::edit_user),
            )
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", openapi.clone()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
