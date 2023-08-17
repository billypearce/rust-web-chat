use serde::Deserialize;
use std::fs;
use axum::{
    routing::get,
    response::{Html, Redirect},
    Router,
    extract::{Path, Form},
    http::header::{HeaderMap, CONTENT_TYPE},
};
use rusqlite::Connection;

pub async fn run() {

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/:id", get(home))
        .route("/login", get(login_page).post(auth))
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
    let document = fs::read_to_string("auth/login.html").unwrap();
    Html(document)
}

#[axum_macros::debug_handler]
async fn auth(Form(creds): Form<UserCredentials>) -> Redirect {
    let db = Connection::open("users.db").unwrap();

    dbg!(&creds);
    let UserCredentials { username, password } = creds;

    let mut stmt = db.prepare("SELECT rowid FROM users WHERE username = ?1 AND password = ?2").unwrap();
    let result = stmt.query_row([username, password], |row| {
        let id: i32 = row.get(0).unwrap();
        Ok(id)
    });

    match result {
        Ok(id) => Redirect::to(format!("/{}", id).as_str()),
        Err(_) => Redirect::to("/login"),
    }
}

async fn register_page() -> Html<String> {
    let document = fs::read_to_string("auth/register.html").unwrap();
    Html(document)
}

async fn create_user(Form(creds): Form<UserCredentials>) -> Redirect {
    let db = Connection::open("users.db").unwrap();

    dbg!(&creds);
    let UserCredentials { username, password } = creds;

    let result = db.execute("INSERT INTO users VALUES (?1, ?2)", (username, password));

    match result {
        Ok(_) => Redirect::to("/login"),
        Err(_) => Redirect::to("/register"),
    }
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

pub enum StaticFileType {
    Css,
    Js,
    Img,
    Other,
}

pub fn extension(filename: &str) -> StaticFileType {
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

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use crate::{extension, StaticFileType};

    #[test]
    fn css_file_type() {
        assert!(matches!(extension("ahhhHHHHHHAHAHHHHH_awewrjwlejrlwjerj__bruhhhh.css"), StaticFileType::Css));
    }
}