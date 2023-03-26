use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};
use aws_sdk_s3 as s3;
use aws_sdk_s3::Endpoint;
use devcade_api_rs::{
    games::{self, FileUploadDoc, GameData, GameUploadDoc},
    models::{AppState, Game},
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
        ),
        components(
            schemas(GameData, Game, GameUploadDoc, FileUploadDoc)
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
            "https://s3.csh.rit.edu".parse().unwrap(),
        ))
        //.endpoint_resolver(s3::Endpoint::immutable("https://s3.csh.rit.edu".parse().unwrap()))
        .build();
    let s3_conn = s3::Client::from_conf(s3_config);

    let pool = PgPoolOptions::new()
        .connect(&env::var("SQL_URI").unwrap())
        .await
        .unwrap();
    HttpServer::new(move || {
        App::new()
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
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", openapi.clone()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
