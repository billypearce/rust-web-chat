use axum::{
    extract::{
        ws::{Message, WebSocket}, Extension, Form, Path, Query, State, WebSocketUpgrade
    },
    http::header::{HeaderMap, CONTENT_TYPE},
    response::{Html, Redirect, Response},
};
use minijinja::{context, Environment};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::Value;
use std::fs;

use crate::staticfiles::{extension, StaticFileType};

pub async fn home(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("index").expect("Template not found");
    let text = match template.render(context!()) {
        Ok(text) => text,
        Err(_) => String::from("Template rendering error."),
    };
    Html(text)
}

pub async fn login_page(Query(status): Query<AuthResult>, Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("login").unwrap();

    let failed = match status.fail {
        Some(b) => b,
        None => false,
    };

    Html(template.render(context!{fail => failed}).unwrap())
}

#[axum_macros::debug_handler]
pub async fn auth(Form(creds): Form<UserCredentials>) -> Redirect {
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
        Ok(id) => Redirect::to(format!("/{}", id).as_str()),
        Err(_) => Redirect::to("/login?fail=true"),
    }
}

pub async fn register_page(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("register").unwrap();
    Html(template.render(context!()).unwrap())
}

pub async fn create_user(Form(creds): Form<UserCredentials>) -> Redirect {
    let db = Connection::open("users.db").unwrap();

    dbg!(&creds);
    let UserCredentials { username, password } = &creds;

    let result = db.execute("INSERT INTO users VALUES (?1, ?2)", (username, password));

    match result {
        Ok(_) => auth(Form(creds)).await,
        Err(_) => Redirect::to("/register?fail=true"),
    }
}

#[axum_macros::debug_handler]
pub async fn static_file(Path(path): Path<String>) -> (HeaderMap, Vec<u8>) {
    let file = fs::read("static/".to_owned() + &path);
    let mut header_map = HeaderMap::new();

    match file {
        Ok(file) => {

            match extension(&path) {
                StaticFileType::Css => header_map.insert(CONTENT_TYPE, "text/css".parse().unwrap()),
                StaticFileType::Js => header_map.insert(CONTENT_TYPE, "text/javascript".parse().unwrap()),
                StaticFileType::Img => panic!("images not supported yet."),
                StaticFileType::Font => header_map.insert(CONTENT_TYPE, "font/ttf".parse().unwrap()),
                StaticFileType::Other => panic!("invalid file type requested."),
            };

            (header_map, file)
        },
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("{}\nRequested resource not found: {}", e, &path);
                },
                _ => {
                    eprintln!("Error retrieving resource: {}", e);
                }
            }
            (header_map, Vec::new())
        }
    }

}

pub async fn echo(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_echo)
}

async fn handle_echo(mut sock: WebSocket) {
    while let Some(msg) = sock.recv().await {
        let msg = if let Ok(msg) = msg {
            let msg = extract_message(msg).unwrap(); // TODO refactor this trash
            let msg = strip_quotes(&msg);
            format!(
                "<div id='chat-box' hx-swap-oob='beforeend'><span class='display-name'>NAEM:</span> {}<br></div>",
                msg
            )
        } else {
            // client dc
            return;
        };

        if sock.send(Message::Text(msg)).await.is_err() {
            // client dc
            return;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthResult {
    pub fail: Option<bool>,
}

fn extract_message(raw_message: Message) -> Result<String, serde_json::Error> {
    let Message::Text(msg_str) = raw_message else {
        return Ok("".to_string());
    }; // TODO actual error handling
    let json: Value = serde_json::from_str(&msg_str)?;
    let value = &json["msg"];
    let msg = serde_json::to_string(&value)?;
    Ok(msg.clone())
}

// This function assumes the input was extracted from raw json,
// and therefore has quotes at the beginning and end
// it does not search for quotes characters
fn strip_quotes(string: &str) -> &str {
    let length = string.len();
    &string[1..length - 1]
}
