#[cfg(test)]
use crate::tests::{get_test_server, MCDADE_USER, MTFT_USER};

#[actix_web::test]
async fn test_get_user() {
    let srv = get_test_server().await;
    let req = srv.get("/users/skyz");
    let mut res = req.send().await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_add_user() {
    let srv = get_test_server().await;
    let req = srv
        .post("/users/")
        .insert_header(("frontend_api_key", "TESTING"));
    let mut res = req.send_json(&MCDADE_USER.clone()).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_add_user_unauthorized() {
    let srv = get_test_server().await;
    let req = srv.post("/users/");
    let mut res = req.send_json(&MCDADE_USER.clone()).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_edit_user() {
    let srv = get_test_server().await;
    let req = srv
        .put(format!("/users/{}", MTFT_USER.id))
        .insert_header(("frontend_api_key", "TESTING"));
    let mut edited_user = MTFT_USER.clone();
    edited_user.picture = "CHANGE PICTURE".to_string();
    let mut res = req.send_json(&edited_user).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert!(res.status().is_success());
}

#[actix_web::test]
async fn test_edit_user_unauthorized() {
    let srv = get_test_server().await;
    let req = srv.put(format!("/users/{}", MTFT_USER.id));
    let mut edited_user = MTFT_USER.clone();
    edited_user.picture = "CHANGE PICTURE".to_string();
    let mut res = req.send_json(&edited_user).await.unwrap();
    println!(
        "{} | {}",
        res.status(),
        std::str::from_utf8(&res.body().await.unwrap()).unwrap_or("Unreadable")
    );
    assert_eq!(res.status().as_u16(), 401);
}
