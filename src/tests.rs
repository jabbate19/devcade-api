use crate::{
    games::routes::{self as games, FileUploadDoc, GameData, GameUploadDoc},
    models::{AppState, Game, GameWithTags, Tag, User, UserType},
    tags::routes as tags,
    users::routes as users,
};
use actix_test::TestServer;
use actix_web::{
    web::{self, scope, Data},
    App,
};
use aws_sdk_s3 as s3;
use aws_sdk_s3::Endpoint;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use sqlx::postgres::PgPoolOptions;
use std::env;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

lazy_static! {
    pub static ref TEST_GAME_A: Game = make_test_game(
        "A",
        "skyz",
        "5ec8f244899431af8effad9e7ec9b2543226c78f",
        NaiveDate::from_ymd_opt(2023, 03, 23).unwrap(),
        "TestGameA"
    );
    pub static ref TEST_GAME_B: Game = make_test_game(
        "B",
        "qel",
        "7b91c68006227a64c3cabcb6aa67f3e3e792b11b",
        NaiveDate::from_ymd_opt(2023, 03, 23).unwrap(),
        "TestGameB"
    );
    pub static ref TEST_GAME_C: Game = make_test_game(
        "C",
        "ella",
        "0d641140903c21d47a007b0136e7c2a7295a254a",
        NaiveDate::from_ymd_opt(2023, 03, 23).unwrap(),
        "TestGameC"
    );
    pub static ref TEST_GAME_D: Game = make_test_game(
        "D",
        "atom",
        "04d6c7defa5dd48067cb44a473ac8eeb17f529f5",
        NaiveDate::from_ymd_opt(2023, 03, 23).unwrap(),
        "TestGameD"
    );
    pub static ref TEST_GAME_E: Game = make_test_game(
        "E",
        "joeneil",
        "5d4ac1284877c9262df5808b8ab0e922863f9464",
        NaiveDate::from_ymd_opt(2023, 03, 23).unwrap(),
        "TestGameE"
    );
    pub static ref TEST_TAG_1: Tag = Tag {
        name: "TestTag1".to_string(),
        description: "TestTag1 Description".to_string(),
    };
    pub static ref TEST_TAG_2: Tag = Tag {
        name: "TestTag2".to_string(),
        description: "TestTag2 Description".to_string(),
    };
    pub static ref TEST_TAG_3: Tag = Tag {
        name: "TestTag3".to_string(),
        description: "TestTag3 Description".to_string(),
    };
    pub static ref TEST_TAG_4: Tag = Tag {
        name: "TestTag4".to_string(),
        description: "TestTag4 Description".to_string(),
    };
    pub static ref TEST_TAG_5: Tag = Tag {
        name: "TestTag5".to_string(),
        description: "TestTag5 Description".to_string(),
    };
    pub static ref TEST_TAG_6: Tag = Tag {
        name: "TestTag6".to_string(),
        description: "TestTag6 Description".to_string(),
    };
    pub static ref TEST_TAG_7: Tag = Tag {
        name: "TestTag7".to_string(),
        description: "TestTag7 Description".to_string(),
    };
    pub static ref TEST_TAG_8: Tag = Tag {
        name: "TestTag8".to_string(),
        description: "TestTag8 Description".to_string(),
    };
    pub static ref TEST_TAG_9: Tag = Tag {
        name: "TestTag9".to_string(),
        description: "TestTag9 Description".to_string(),
    };
    pub static ref TEST_TAG_10: Tag = Tag {
        name: "TestTag10".to_string(),
        description: "TestTag10 Description".to_string(),
    };
    pub static ref TEST_TAG_11: Tag = Tag {
        name: "TestTag11".to_string(),
        description: "TestTag11 Description".to_string(),
    };
    pub static ref TEST_TAG_12: Tag = Tag {
        name: "TestTag12".to_string(),
        description: "TestTag12 Description".to_string(),
    };
    pub static ref TEST_TAG_13: Tag = Tag {
        name: "TestTag13".to_string(),
        description: "TestTag13 Description".to_string(),
    };
    pub static ref TEST_TAG_14: Tag = Tag {
        name: "TestTag14".to_string(),
        description: "TestTag14 Description".to_string(),
    };
    pub static ref TEST_TAG_15: Tag = Tag {
        name: "TestTag15".to_string(),
        description: "TestTag15 Description".to_string(),
    };
    pub static ref TEST_TAG_16: Tag = Tag {
        name: "TestTag16".to_string(),
        description: "TestTag16 Description".to_string(),
    };
    pub static ref TEST_TAG_17: Tag = Tag {
        name: "TestTag17".to_string(),
        description: "TestTag17 Description".to_string(),
    };
    pub static ref TEST_TAG_18: Tag = Tag {
        name: "TestTag18".to_string(),
        description: "TestTag18 Description".to_string(),
    };
    pub static ref TEST_TAG_19: Tag = Tag {
        name: "TestTag19".to_string(),
        description: "TestTag19 Description".to_string(),
    };
    pub static ref TEST_TAG_20: Tag = Tag {
        name: "TestTag20".to_string(),
        description: "TestTag20 Description".to_string(),
    };
    pub static ref SKYZ_USER: User = User::from_csh("skyz", "Joe", "Abbate", true);
    pub static ref QEL_USER: User = User::from_csh("qel", "Jeremy", "Smart", false);
    pub static ref ELLA_USER: User = User::from_csh("ella", "Ella", "Soccoli", false);
    pub static ref ATOM_USER: User = User::from_csh("atom", "Ata", "Noor", false);
    pub static ref JOENEIL_USER: User = User::from_csh("joeneil", "Joe", "ONeil", true);
    pub static ref MTFT_USER: User = User::from_csh("mtft", "Noah", "Emke", true);
    pub static ref DANI_USER: User = User::from_csh("nintendods", "Dani", "Saba", false);
    pub static ref JAYH_USER: User = User::from_csh("jayh", "Jay", "Horsfall", false);
    pub static ref GOD_USER: User = User::from_csh("god", "Brett", "Huber", false);
    pub static ref ROSE_USER: User = User::from_csh("rose", "Johanna", "Wichmann", false);
    pub static ref ATLAS_USER: User = User::from_csh("atlas", "Emma", "Schmitt", false);
    pub static ref WILNIL_USER: User = User::from_csh("wilnil", "Willard", "Nilges", false);
    pub static ref LOG_USER: User = User::from_csh("log", "Logan", "Endes", false);
    pub static ref TACOTUESDAY_USER: User =
        User::from_csh("tacotuesday", "Conner", "Meagher", false);
    pub static ref BIGC_USER: User = User::from_csh("bigc", "Connor", "Langa", false);
    pub static ref EVAN_USER: User = User::from_csh("evan", "Evan", "Clough", false);
    pub static ref OTTO_USER: User = User::from_csh("otto", "Marcus", "Otto", false);
    pub static ref TEX_USER: User = User::from_csh("tex", "Avan", "Peltier", false);
    pub static ref PDNTSPA_USER: User = User::from_csh("PDNTSPA", "Curtis", "Heater", false);
    pub static ref SULTANOFSWING_USER: User =
        User::from_csh("sultanofswing", "Charlie", "Salinetti", false);
    pub static ref SAMC_USER: User = User::from_csh("samc", "Sam", "Cordry", false);
    pub static ref FISH_USER: User = User::from_csh("fish", "Nate", "Aquino", false);
    pub static ref LIMABEAN_USER: User = User::from_csh("limabean", "Darwin", "Tran", false);
    pub static ref BABYSATAN_USER: User = User::from_csh("babysatan", "Alex", "Vasilcoiu", false);
    pub static ref THEAI_USER: User = User::from_csh("theai", "Ada", "Foster", false);
    pub static ref SIHANG_USER: User = User::from_csh("sihang", "Sihang", "Hu", false);
    pub static ref MCDADE_USER: User = User::from_csh("mcdade", "Wilson", "McDade", true);
    pub static ref TEST_GAME_A_WITH_TAGS: GameWithTags = GameWithTags::new(
        TEST_GAME_A.clone(),
        vec![TEST_TAG_1.clone()],
        SKYZ_USER.clone()
    );
    pub static ref TEST_GAME_B_WITH_TAGS: GameWithTags =
        GameWithTags::new(TEST_GAME_B.clone(), vec![], QEL_USER.clone());
}

fn make_test_game(id_letter: &str, author: &str, hash: &str, date: NaiveDate, name: &str) -> Game {
    Game {
        id: format!(
            "{}-{}-{}-{}-{}",
            id_letter.repeat(8),
            id_letter.repeat(4),
            id_letter.repeat(4),
            id_letter.repeat(4),
            id_letter.repeat(12)
        ),
        author: author.to_string(),
        upload_date: date,
        name: name.to_string(),
        hash: hash.to_string(),
        description: format!("{} Description", name),
    }
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
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
    cfg.service(
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
    .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", openapi.clone()));
}

pub async fn get_app_data() -> Data<AppState> {
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
    Data::new(AppState {
        db: pool.clone(),
        s3: s3_conn.clone(),
    })
}

pub async fn get_test_server() -> TestServer {
    let app_data = get_app_data().await;
    actix_test::start(move || {
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone())
    })
}

#[actix_web::test]
async fn test_docs_reachable() {
    let srv = get_test_server().await;
    let req = srv.get("/docs/");
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_openapi_reachable() {
    let srv = get_test_server().await;
    let req = srv.get("/api-doc/openapi.json");
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}
