use axum::{
    extract::{Extension, Path},
    response::Html
};
use minijinja::{context, Environment};
use rusqlite::Connection;

// re-export of handlers from other modules
pub use crate::{
    staticfiles::static_file,
    auth::{auth, login_page, register_page, create_user},
    chat::chat,
};

pub async fn home(Extension(env): Extension<Environment<'_>>, Path(userid): Path<u32>) -> Html<String> {
    let template = env.get_template("index").expect("Template not found");

    let db = Connection::open("users.db").unwrap();

    let mut stmt = db
        .prepare("SELECT username FROM users WHERE rowid = ?1")
        .unwrap();
    let result = stmt.query_row([userid], |row| {
        let name: String = match row.get(0) {
            Ok(name) => name,
            Err(_) => String::from("invalid_user"),
        };
        Ok(name)
    });

    let text = match template.render(context!{name => result.unwrap()}) {
        Ok(text) => text,
        Err(_) => String::from("Template rendering error."),
    };
    Html(text)
}
