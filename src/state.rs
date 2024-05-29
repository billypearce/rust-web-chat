use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::extract::ws::WebSocket;

#[derive(Clone)]
pub struct TopLevelState {
    pub state: Arc<Mutex<AppState>>,
}

pub struct AppState {
    rooms: HashMap<String, Room>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            rooms: HashMap::<String, Room>::new(),
        }
    }

    pub fn create_room(&mut self, name: String) {
        let room = Room::new(name.clone());

        self.rooms.insert(name, room);
    }
}

pub struct Room {
    name: String,
    users: HashMap<String, WebSocket>,
}

impl Room {
    pub fn new(name: String) -> Room {
        Room {
            name,
            users: HashMap::<String, WebSocket>::new(),
        }
    }
}