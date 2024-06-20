use axum::{
    extract::Extension,
    response::Html
};
use minijinja::{context, Environment};

pub use crate::{
    staticfiles::static_file,
    auth::{auth, login_page, register_page, create_user},
    chat::chat,
};

pub async fn home(Extension(env): Extension<Environment<'_>>) -> Html<String> {
    let template = env.get_template("index").expect("Template not found");
    let text = match template.render(context!()) {
        Ok(text) => text,
        Err(_) => String::from("Template rendering error."),
    };
    Html(text)
}
