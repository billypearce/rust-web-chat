mod handlers;
mod staticfiles;
mod state;

use axum::{extract::Extension, response::Redirect, routing::get, Router};
use minijinja::Environment;
use rusqlite::Connection;
use state::AppState;
use std::{
    sync::{Arc, Mutex},
    fs, path::Path
};

use crate::state::TopLevelState;

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

    let state = TopLevelState{
        state: Arc::new(Mutex::new(AppState::new()))
    };

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/:id", get(handlers::home))
        .route("/login", get(handlers::login_page).post(handlers::auth))
        .route(
            "/register",
            get(handlers::register_page).post(handlers::create_user),
        )
        .route("/static/*path", get(handlers::static_file))
        .route("/echo", get(handlers::echo))
        .layer(Extension(env))
        .with_state(state);

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
        Err(e) => panic!("DB oopsie: {}", e),
    }
}
