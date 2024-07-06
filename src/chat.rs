use axum::{
    extract::{ws::{Message, WebSocket}, Path, State, WebSocketUpgrade
    }, http::HeaderMap, response::Response, Extension, response::Html
};
use tokio::sync::broadcast::{Sender, Receiver};
use serde_json::Value;
use futures::{stream::StreamExt, sink::SinkExt};
use minijinja::{Environment, context};
use rusqlite::Connection;

use crate::AppState;

#[axum_macros::debug_handler]
pub async fn chat_page(
    Extension(env): Extension<Environment<'static>>, 
    headers: HeaderMap,
) -> Html<String> {
    let userid = headers.get("cookie");

    let userid = userid.unwrap().to_str().unwrap();
    let split: Vec<&str> = userid.split("=").collect();
    let userid = split[1];
    let userid: i32 = userid.parse().unwrap();

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

    let text = match result {
        Ok(name) => {
            match template.render(context!{name}) {
                Ok(text) => text,
                Err(_) => String::from("Template rendering error."),
            }
        },
        Err(e) => {
            format!("error: {}", e)
        }
    };

    Html(text)
}

pub async fn chat(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>,
    Path(name): Path<String>
) -> Response {
    let (tx, rx) = state.channel.subscribe();
    ws.on_upgrade(|ws| handle_chat(ws, tx, rx, name))
}

async fn handle_chat(
    sock: WebSocket,
    tx: Sender<String>, 
    mut rx: Receiver<String>,
    name: String
) {
    let (sock_tx, sock_rx) = sock.split();

    tx.send(format!(
        "<div id='chat-box' hx-swap-oob='beforeend'><span class='display-name'>{} has joined the chat.</span><br></div>", 
        &name
    )).unwrap();

    let mut send_task = tokio::spawn(async move {
        let mut sock = sock_tx;

        while let Ok(msg) = rx.recv().await {
            match sock.send(Message::Text(msg)).await {
                Ok(_) => (),
                Err(_) => return,
            };
        }
    });

    let display_name = name.clone();
    let task_tx = tx.clone();

    let mut recv_task = tokio::spawn(async move {
        let mut sock = sock_rx;

        while let Some(Ok(msg)) = sock.next().await {
            let msg = extract_message(msg).unwrap();
            let msg = strip_quotes(&msg);
            let msg = format!("<div id='chat-box' hx-swap-oob='beforeend'><span class='display-name'>{}:</span> {}<br></div>", display_name, msg);
            if task_tx.send(msg).is_err() {
                return;
            }
        }
    });

    // user dc
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    tx.send(format!(
        "<div id='chat-box' hx-swap-oob='beforeend'><span class='display-name'>{} has left the chat.</span><br></div>", 
        &name
    )).unwrap();
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
