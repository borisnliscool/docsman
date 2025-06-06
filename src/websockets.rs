use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use std::collections::HashMap;
use uuid::Uuid;

pub struct DocsmanSocket {
    socket: WebSocket,
    id: String,
}
pub struct WebsocketManager {
    clients: HashMap<String, DocsmanSocket>,
}

impl Default for WebsocketManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WebsocketManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client: WebSocket) {
        let socket = DocsmanSocket {
            socket: client,
            id: Uuid::new_v4().to_string(),
        };
        self.clients.insert(socket.id.clone(), socket);
    }

    pub fn remove_client(&mut self, id: String) {
        self.clients.remove(&id);
    }

    pub async fn broadcast(&mut self, message: String) {
        let mut to_remove = Vec::new();

        for (id, client) in &mut self.clients {
            if client
                .socket
                .send(Message::Text(Utf8Bytes::from(message.clone())))
                .await
                .is_err()
            {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.remove_client(id);
        }
    }
}
