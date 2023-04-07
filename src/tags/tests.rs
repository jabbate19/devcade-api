use crate::models::Tag;
#[cfg(test)]
use crate::tests::{get_test_server, TEST_TAG_1, TEST_TAG_2, TEST_TAG_3, TEST_TAG_4, TEST_TAG_6};

#[actix_web::test]
async fn test_get_all_tags() {
    let srv = get_test_server().await;
    let req = srv.get("/tags/");
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_get_tag() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/tags/{}", TEST_TAG_1.name));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_tag() {
    let srv = get_test_server().await;
    let mut edited_tag = TEST_TAG_2.clone();
    edited_tag.name = "NEWNAME2".to_string();
    edited_tag.description = "I changed the description!".to_string();
    let req = srv
        .put(format!("/tags/{}", TEST_TAG_2.name))
        .insert_header(("frontend_api_key", "TESTING"));
    let mut res = req.send_json(&edited_tag).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_tag_unauthorized() {
    let srv = get_test_server().await;
    let mut edited_tag = TEST_TAG_2.clone();
    edited_tag.name = "NEWNAME2".to_string();
    edited_tag.description = "I changed the description!".to_string();
    let req = srv.put(format!("/tags/{}", TEST_TAG_2.name));
    let mut res = req.send_json(&edited_tag).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_delete_tag() {
    let srv = get_test_server().await;
    let req = srv
        .delete(format!("/tags/{}", TEST_TAG_3.name))
        .insert_header(("frontend_api_key", "TESTING"));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_delete_tag_unauthorized() {
    let srv = get_test_server().await;
    let req = srv.delete(format!("/tags/{}", TEST_TAG_3.name));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_add_tag() {
    let srv = get_test_server().await;
    let req = srv
        .post("/tags/")
        .insert_header(("frontend_api_key", "TESTING"));
    let new_tag = Tag {
        name: "NEW_TAG".to_string(),
        description: "THIS IS A NEW TAG".to_string(),
    };
    let mut res = req.send_json(&new_tag).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_add_tag_unauthorized() {
    let srv = get_test_server().await;
    let req = srv.post("/tags/");
    let res = req.send_json(&TEST_TAG_6.clone()).await.unwrap();
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_get_tag_games_none() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/tags/{}/games", TEST_TAG_4.name));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_get_tag_games_some() {
    let srv = get_test_server().await;
    let req = srv.get(format!("/tags/{}/games", TEST_TAG_1.name));
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}
