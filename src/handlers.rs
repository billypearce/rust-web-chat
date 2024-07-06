use axum::{
    http::HeaderMap, response::Redirect
};
use rusqlite::Connection;

// re-export of handlers from other modules
pub use crate::{
    staticfiles::static_file,
    auth::{auth, login_page, register_page, create_user},
    chat::{chat_page, chat},
};

pub async fn home(
    headers: HeaderMap
) -> Redirect {
    let userid = headers.get("cookie");

    if userid.is_none() {
        return Redirect::to("/login");
    }

    let userid = userid.unwrap().to_str().unwrap();
    let split: Vec<&str> = userid.split("=").collect();
    let userid = split[1];
    let userid: i32 = userid.parse().unwrap();

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

    match result {
        Ok(_) => Redirect::to("/home"),
        Err(_) => Redirect::to("/login"),
    }
}
