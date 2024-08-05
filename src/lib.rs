mod handlers;
mod staticfiles;
mod state;
mod auth;
mod chat;

use axum::{extract::Extension, routing::get, Router};
use minijinja::Environment;
use rusqlite::Connection;
use std::{
    fs, 
    path::Path
};

use crate::{
    handlers::{home, static_file, login_page, auth, register_page, create_user, chat_page, chat},
    state::AppState,
};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    env.add_template_owned("base", fs::read_to_string("templates/base.html")?)?;
    env.add_template_owned("index", fs::read_to_string("templates/index.html")?)?;
    env.add_template_owned("login", fs::read_to_string("templates/auth/login.html")?)?;
    env.add_template_owned(
        "register",
        fs::read_to_string("templates/auth/register.html")?,
    )?;

    init_db();

    let state = AppState::with_capacity(16);

    let app = Router::new()
        .route("/", get(home))
        .route("/home", get(chat_page))
        .route("/login", get(login_page).post(auth))
        .route(
            "/register",
            get(register_page).post(create_user),
        )
        .route("/static/*path", get(static_file))
        .route("/chat/:name", get(chat))
        .layer(Extension(env))
        .with_state(state);

    println!("Server running: http://localhost:3000/");

    axum::Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn init_db() {
    match Path::new("users.db").try_exists() {
        Ok(true) => (),
        Ok(false) => {
            let db = Connection::open("users.db").unwrap();
            db.execute("CREATE TABLE users(username text UNIQUE, password text);", ()).unwrap();
        },
        Err(e) => panic!("DB error: {}", e),
    }
}
