use axum::{
    extract::{WebSocketUpgrade, State, ws::{WebSocket, Message},
    },
    response::Response,
};
use tokio::sync::broadcast::{Sender, Receiver};
use serde_json::Value;
use futures::{stream::StreamExt, sink::SinkExt};

use crate::AppState;

pub async fn chat(
    ws: WebSocketUpgrade, 
    State(state): State<AppState>
) -> Response {
    let (tx, rx) = state.channel.subscribe();
    ws.on_upgrade(|ws| handle_chat(ws, tx, rx))
}

async fn handle_chat(
    sock: WebSocket,
    tx: Sender<String>, 
    mut rx: Receiver<String>
) {
    let (sock_tx, sock_rx) = sock.split();

    let _send_task = tokio::spawn(async move {
        let mut sock = sock_tx;

        while let Ok(msg) = rx.recv().await {
            match sock.send(Message::Text(msg)).await {
                Ok(_) => (),
                Err(_) => return,
            };
        }
    });

    let _recv_task = tokio::spawn(async move {
        let mut sock = sock_rx;

        while let Some(Ok(msg)) = sock.next().await {
            let msg = extract_message(msg).unwrap();
            let msg = strip_quotes(&msg);
            let msg = format!("<div id='chat-box' hx-swap-oob='beforeend'><span class='display-name'>NAEM:</span> {}<br></div>", msg);
            if tx.send(msg).is_err() {
                return;
            }
        }
    });
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
