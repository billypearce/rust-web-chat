use serde::Deserialize;
use std::fs;
use axum::{
    routing::get,
    response::{Html, Redirect},
    Router,
    extract::{Path, Form, Extension},
    http::header::{HeaderMap, CONTENT_TYPE},
};
use rusqlite::Connection;
use minijinja::{Environment, context};

pub async fn run() {

    let mut env = Environment::new();
    env.add_template_owned("base", fs::read_to_string("templates/base.html").unwrap()).unwrap();
    env.add_template_owned("index", fs::read_to_string("templates/index.html").unwrap()).unwrap();
    env.add_template_owned("login", fs::read_to_string("templates/auth/login.html").unwrap()).unwrap();
    env.add_template_owned("register", fs::read_to_string("templates/auth/register.html").unwrap()).unwrap();

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/:id", get(home))
        .route("/login", get(login_page).post(auth))
        .route("/register", get(register_page).post(create_user))
        .route("/static/*path", get(static_file))
        .layer(Extension(env));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}

async fn home(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("index").unwrap();
    Html(template.render(context!()).unwrap())
}

async fn login_page(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("login").unwrap();
    Html(template.render(context!()).unwrap())
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

async fn register_page(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("register").unwrap();
    Html(template.render(context!()).unwrap())
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