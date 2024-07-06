use axum::{
    extract::{Extension, Form, Query}, http::{header::SET_COOKIE, HeaderMap}, response::{Html, Redirect}
};
use rusqlite::Connection;
use minijinja::{Environment, context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthResult {
    pub fail: Option<bool>,
}

pub async fn login_page(
    Query(status): Query<AuthResult>, 
    Extension(env): Extension<Environment<'_>>
) -> Html<String> {
    let template = env.get_template("login").unwrap();

    let failed = match status.fail {
        Some(b) => b,
        None => false,
    };

    Html(template.render(context!{fail => failed}).unwrap())
}

#[axum_macros::debug_handler]
pub async fn auth(
    Form(creds): Form<UserCredentials>
) -> (HeaderMap, Redirect) {
    let db = Connection::open("users.db").unwrap();

    dbg!(&creds);
    let UserCredentials { username, password } = creds;

    let mut stmt = db
        .prepare("SELECT rowid FROM users WHERE username = ?1 AND password = ?2")
        .unwrap();

    let result = stmt.query_row([username, password], |row| {
        let id: i32 = row.get(0).unwrap();
        Ok(id)
    });

    match result {
        Ok(id) => {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, format!("id={}", id).parse().unwrap());

            (headers, Redirect::to("/"))
        },
        Err(_) => {
            (HeaderMap::new(), Redirect::to("/login?fail=true"))
        }
    }
}

pub async fn register_page(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("register").unwrap();
    Html(template.render(context!()).unwrap())
}

pub async fn create_user(Form(creds): Form<UserCredentials>) -> (HeaderMap, Redirect) {
    let db = Connection::open("users.db").unwrap();

    dbg!(&creds);
    let UserCredentials { username, password } = &creds;

    let result = db.execute("INSERT INTO users VALUES (?1, ?2)", (username, password));

    match result {
        Ok(_) => auth(Form(creds)).await,
        Err(_) => (HeaderMap::new(), Redirect::to("/register?fail=true")),
    }
}
