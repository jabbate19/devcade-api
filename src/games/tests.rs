use std::{fs::File, io::Read};

use crate::app::{configure_app, get_app_data};
#[cfg(test)]
use crate::{
    models::GameWithTags,
    tests::{
        get_test_server, TEST_GAME_A, TEST_GAME_A_WITH_TAGS, TEST_GAME_B, TEST_GAME_B_WITH_TAGS,
        TEST_GAME_C, TEST_GAME_D, TEST_GAME_E,
    },
};

use actix_web::{test, App};

#[derive(Debug)]
pub struct GameUploadTest {
    pub game: File,
    pub banner: File,
    pub icon: File,
    pub title: String,
    pub description: String,
    pub author: String,
}

impl GameUploadTest {
    pub fn to_payload(&mut self, boundary: &str) -> Vec<u8> {
        let mut out_str: String = String::from("");
        out_str.push_str(boundary);
        out_str.push('\r');
        out_str.push('\n');
        out_str.push_str(&format!(
            "Content-Disposition: form-data; name=\"title\"\r\n\r\n{}\r\n",
            self.title
        ));
        out_str.push_str(boundary);
        out_str.push('\r');
        out_str.push('\n');
        out_str.push_str(&format!(
            "Content-Disposition: form-data; name=\"description\"\r\n\r\n{}\r\n",
            self.description
        ));
        out_str.push_str(boundary);
        out_str.push('\r');
        out_str.push('\n');
        out_str.push_str(&format!(
            "Content-Disposition: form-data; name=\"author\"\r\n\r\n{}\r\n",
            self.author
        ));

        out_str.push_str(boundary);
        out_str.push('\r');
        out_str.push('\n');
        out_str.push_str("Content-Disposition: form-data; name=\"game\"; filename=\"game.zip\"\r\nContent-Type: application/zip\r\n\r\n");
        let mut out_vec: Vec<u8> = Vec::from(out_str.as_bytes());
        let game_file = &mut self.game;
        let _ = game_file.read_to_end(&mut out_vec);
        let x = format!(
            "\r\n\r\n{}\r\nContent-Disposition: form-data; name=\"banner\"; filename=\"banner\"\r\nContent-Type: image/png\r\n\r\n",
            boundary
        );
        out_vec.append(&mut Vec::from(x.as_bytes()));
        let banner_file = &mut self.banner;
        let _ = banner_file.read_to_end(&mut out_vec);
        let x = format!(
            "\r\n\r\n{}\r\nContent-Disposition: form-data; name=\"icon\"; filename=\"icon\"\r\nContent-Type: image/png\r\n\r\n",
            boundary
        );
        out_vec.append(&mut Vec::from(x.as_bytes()));
        let icon_file = &mut self.icon;
        let _ = icon_file.read_to_end(&mut out_vec);
        let x = format!("\r\n\r\n{}--", boundary);
        out_vec.append(&mut Vec::from(x.as_bytes()));
        out_vec
    }
}

#[derive(Debug)]
pub struct FileUploadTest {
    pub file: File,
}

impl FileUploadTest {
    pub fn to_payload(&mut self, boundary: &str, mimetype: &str) -> Vec<u8> {
        let mut out_str: String = String::from("");
        out_str.push_str(boundary);
        out_str.push('\r');
        out_str.push('\n');
        out_str.push_str(&format!(
            "Content-Disposition: form-data; name=\"file\"\r\nContent-Type: {}\r\n\r\n",
            mimetype
        ));
        let mut out_vec: Vec<u8> = Vec::from(out_str.as_bytes());
        let file = &mut self.file;
        let _ = file.read_to_end(&mut out_vec);
        let x = format!("\r\n\r\n{}--", boundary);
        out_vec.append(&mut Vec::from(x.as_bytes()));
        out_vec
    }
}

#[actix_web::test]
async fn test_get_all_games() {
    let srv = get_test_server().await;
    let req = srv.get("/games/");
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_get_game_no_tags() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/games/{}", TEST_GAME_B.id));
    let mut res = req.send().await.unwrap();
    assert!(res.status().is_success());
    let game_data: GameWithTags = res.json::<GameWithTags>().await.unwrap();
    assert_eq!(game_data, *TEST_GAME_B_WITH_TAGS);
}

#[actix_web::test]
async fn test_get_game_with_tags() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/games/{}", TEST_GAME_A.id));
    let mut res = req.send().await.unwrap();
    assert!(res.status().is_success());
    let game_data: GameWithTags = res.json::<GameWithTags>().await.unwrap();
    assert_eq!(game_data, *TEST_GAME_A_WITH_TAGS);
}

#[actix_web::test]
async fn test_edit_game_data() {
    let srv = get_test_server().await;
    let mut edited_game = TEST_GAME_C.clone();
    edited_game.name = "I changed the name!".to_string();
    edited_game.description = "I changed the description!".to_string();
    let req = srv
        .put(format!("/games/{}", edited_game.id))
        .insert_header(("frontend_api_key", "TESTING"));
    let mut res = req.send_json(&edited_game).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_data_unauthorized() {
    let srv = get_test_server().await;
    let mut edited_game = TEST_GAME_C.clone();
    edited_game.name = "I changed the name!".to_string();
    edited_game.description = "I changed the description!".to_string();
    let req = srv.put(format!("/games/{}", edited_game.id));
    let mut res = req.send_json(&edited_game).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_delete_game() {
    let srv = get_test_server().await;
    let req = srv
        .delete(format!("/games/{}", TEST_GAME_D.id))
        .insert_header(("frontend_api_key", "TESTING"));
    let res = req.send().await.unwrap();
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_delete_game_unauthorized() {
    let srv = get_test_server().await;
    let req = srv.delete(format!("/games/{}", TEST_GAME_D.id));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_add_game() {
    let gamefile = File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF.zip").unwrap();
    let bannerfile =
        File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/banner").unwrap();
    let iconfile = File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/icon").unwrap();
    let mut gameupload = GameUploadTest {
        game: gamefile,
        banner: bannerfile,
        icon: iconfile,
        title: "Chom".to_string(),
        description: "Chom".to_string(),
        author: "skyz".to_string(),
    };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = gameupload.to_payload("------------------43123453263245325234");
    let req = test::TestRequest::post()
        .uri("/games/")
        .append_header(("frontend_api_key", "TESTING"))
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_add_game_unauthorized() {
    let gamefile = File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF.zip").unwrap();
    let bannerfile =
        File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/banner").unwrap();
    let iconfile = File::open("TESTING/data/FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF/icon").unwrap();
    let mut gameupload = GameUploadTest {
        game: gamefile,
        banner: bannerfile,
        icon: iconfile,
        title: "Chom".to_string(),
        description: "Chom".to_string(),
        author: "skyz".to_string(),
    };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = gameupload.to_payload("------------------43123453263245325234");
    let req = test::TestRequest::post()
        .uri("/games/")
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert_eq!(resp.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_get_game_binary() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/games/{}/game", TEST_GAME_E.id));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_binary() {
    let gamefile = File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH.zip").unwrap();
    let mut fileupload = FileUploadTest { file: gamefile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload =
        fileupload.to_payload("------------------43123453263245325234", "application/zip");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/game")
        .append_header(("frontend_api_key", "TESTING"))
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_binary_unauthorized() {
    let gamefile = File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH.zip").unwrap();
    let mut fileupload = FileUploadTest { file: gamefile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload =
        fileupload.to_payload("------------------43123453263245325234", "application/zip");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/game")
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert_eq!(resp.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_get_game_banner() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/games/{}/banner", TEST_GAME_E.id));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_banner() {
    let bannerfile =
        File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/banner").unwrap();
    let mut fileupload = FileUploadTest { file: bannerfile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = fileupload.to_payload("------------------43123453263245325234", "image/png");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/banner")
        .append_header(("frontend_api_key", "TESTING"))
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_banner_unauthorized() {
    let bannerfile =
        File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/banner").unwrap();
    let mut fileupload = FileUploadTest { file: bannerfile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = fileupload.to_payload("------------------43123453263245325234", "image/png");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/banner")
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert_eq!(resp.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_get_game_icon() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/games/{}/icon", TEST_GAME_E.id));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_icon() {
    let iconfile = File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/icon").unwrap();
    let mut fileupload = FileUploadTest { file: iconfile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = fileupload.to_payload("------------------43123453263245325234", "image/png");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/icon")
        .append_header(("frontend_api_key", "TESTING"))
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_edit_game_icon_unauthorized() {
    let iconfile = File::open("TESTING/data/HHHHHHHH-HHHH-HHHH-HHHH-HHHHHHHHHHHH/icon").unwrap();
    let mut fileupload = FileUploadTest { file: iconfile };
    let app_data = get_app_data().await;
    let app = test::init_service(
        App::new()
            .configure(configure_app)
            .app_data(app_data.clone()),
    )
    .await;
    let payload = fileupload.to_payload("------------------43123453263245325234", "image/png");
    let req = test::TestRequest::put()
        .uri("/games/GGGGGGGG-GGGG-GGGG-GGGG-GGGGGGGGGGGG/icon")
        .append_header((
            "Content-Type",
            "mutlipart/form-data; boundary=----------------43123453263245325234",
        ))
        .append_header(("Content-Length", payload.len()))
        .append_header(("Accept-Encoding", "gzip, deflate, br"))
        .append_header(("Accept", "*/*"))
        .set_payload(payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("{} | {:?}", resp.status(), resp.response().body());
    assert_eq!(resp.status().as_u16(), 401);
}
