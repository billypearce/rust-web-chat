mod handlers;
mod staticfiles;

use axum::{extract::Extension, response::Redirect, routing::get, Router};
use minijinja::Environment;
use std::fs;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = Environment::new();
    env.add_template_owned("base", fs::read_to_string("templates/base.html")?)?;
    env.add_template_owned("index", fs::read_to_string("templates/index.html")?)?;
    env.add_template_owned("login", fs::read_to_string("templates/auth/login.html")?)?;
    env.add_template_owned(
        "register",
        fs::read_to_string("templates/auth/register.html")?,
    )?;

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
        .layer(Extension(env));

    axum::Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
