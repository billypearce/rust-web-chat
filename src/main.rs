use std::{thread, fs};
use axum::{
    routing::get,
    response::Html,
    Router,
    extract::{Path, Json},
    http::{header::{HeaderMap, CONTENT_TYPE}, StatusCode},
    Extension,
};
use sqlx::sqlite::SqlitePool;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("users.db").await.unwrap();
    let app = Router::new()
        .route("/:id", get(home))
        .route("/login", get(login_page).post(auth))
        .layer(Extension(pool))
        .route("/register", get(register_page).post(create_user))
        .route("/foo", get(foo))
        .route("/static/*path", get(static_file));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<String> {
    let document = fs::read_to_string("index.html").unwrap();
    Html(document)
}

async fn login_page() -> Html<String> {
    let document = fs::read_to_string("index.html").unwrap();
    Html(document)
}

#[axum_macros::debug_handler]
async fn auth(pool: Extension<SqlitePool>, Json(payload): Json<UserCredentials>) -> StatusCode {
    dbg!(payload);
    StatusCode::OK
}

async fn register_page() -> Html<String> {
    let document = fs::read_to_string("index.html").unwrap();
    Html(document)
}

async fn create_user() {

}

async fn foo() -> &'static str {
    "foo"
}

#[axum_macros::debug_handler]
async fn static_file(Path(path): Path<String>) -> (HeaderMap, String) {
    let file = fs::read_to_string("static/".to_owned() + &path).unwrap();
    let mut header_map = HeaderMap::new();

    match extension(&path) {
        StaticFileType::Css => header_map.insert(CONTENT_TYPE, "text/css".parse().unwrap()),
        StaticFileType::Js => header_map.insert(CONTENT_TYPE, "text/javascript".parse().unwrap()),
        StaticFileType::Img => panic!("images not supported yet."),
        StaticFileType::Other => panic!("invalid file type requested."),
    };

    (header_map, file)
}

enum StaticFileType {
    Css,
    Js,
    Img,
    Other,
}

fn extension(filename: &str) -> StaticFileType {
    if filename.contains(".css") {
        StaticFileType::Css
    } else if filename.contains(".js") {
        StaticFileType::Js
    } else if filename.contains(".png") || filename.contains(".jpg") {
        StaticFileType::Img
    } else {
        StaticFileType::Other
    }
}

#[derive(Deserialize, Debug)]
struct User {
    id: i32,
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct UserCredentials {
    username: String,
    password: String,
}

#[cfg(test)]
mod tests {
    use crate::{extension, StaticFileType};

    #[test]
    fn css_file_type() {
        assert!(matches!(extension("ahhhHHHHHHAHAHHHHH_awewrjwlejrlwjerj__bruhhhh.css"), StaticFileType::Css));
    }
}