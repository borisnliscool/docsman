use crate::watcher::async_watcher;
use crate::websockets::WebsocketManager;
use crate::DocsmanArguments;
use axum::extract::ws::WebSocket;
use axum::extract::WebSocketUpgrade;
use axum::http::Uri;
use axum::response::{Html, IntoResponse};
use axum::routing::{any, get};
use axum::Router;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures::StreamExt;
use glob::{glob_with, MatchOptions};
use markdown::{CompileOptions, Options};
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::canonicalize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub struct DocsmanServer {
    base_path: PathBuf,
    port: u16,
    host: String,
    layout_content: String,
    websocket_manager: Arc<Mutex<WebsocketManager>>,
    autoreload: bool,
    legend: bool,
}

impl DocsmanServer {
    pub fn new(arguments: DocsmanArguments) -> Self {
        let path = arguments.path.clone();
        let path = if path.is_relative() {
            std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))
                .unwrap()
                .join(path)
        } else {
            path
        };

        Self {
            base_path: path,
            port: arguments.port,
            host: arguments.host,
            layout_content: include_str!("../layout/dist/index.html").to_string(),
            websocket_manager: Arc::new(Mutex::new(WebsocketManager::default())),
            autoreload: arguments.autoreload.unwrap_or(true),
            legend: arguments.legend.unwrap_or(true),
        }
    }

    fn fill_layout(self: Arc<Self>, data_map: HashMap<String, String>) -> String {
        let mut content = self.layout_content.clone();
        for (key, value) in data_map {
            content = content.replace(&format!("%%{}%%", key), &value);
        }
        content
    }

    fn build_path(self: Arc<Self>, path: String) -> Result<PathBuf, String> {
        let final_path = self.base_path.join(path);

        let canonical_final_path = canonicalize(&final_path)
            .map_err(|e| format!("Failed to resolve canonical path: {}", e))?;

        if !canonical_final_path.starts_with(&self.base_path) {
            return Err("Accessing path outside of base directory is not allowed".to_string());
        }

        Ok(canonical_final_path)
    }

    fn build_content(self: Arc<Self>, path: String) -> Result<impl IntoResponse, String> {
        let final_path = self.clone().build_path(path.clone())?;
        let content = std::fs::read_to_string(&final_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let markdown_contents = markdown::to_html_with_options(
            &content,
            &Options {
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    allow_any_img_src: true,
                    ..CompileOptions::default()
                },
                ..Options::default()
            },
        )
        .map_err(|e| format!("Failed to parse markdown: {}", e))?;

        let legend = if self.legend {
            self.clone().get_legend().unwrap_or_default()
        } else {
            Vec::new()
        };

        let data_map = HashMap::from([
            (
                "title".to_string(),
                final_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ),
            (
                "content".to_string(),
                BASE64_STANDARD.encode(markdown_contents),
            ),
            ("page".to_string(), path),
            (
                "legend".to_string(),
                BASE64_STANDARD.encode(
                    serde_json::to_string(&legend)
                        .map_err(|e| format!("Failed to serialize legend: {}", e))?,
                ),
            ),
        ]);

        let content = self.fill_layout(data_map);
        Ok(Html(content))
    }

    fn root_handler(self: Arc<Self>) -> Result<impl IntoResponse, String> {
        self.build_content("index.md".to_string())
    }

    async fn on_file_update(self: Arc<Self>, path: String) {
        let mut manager = self.websocket_manager.lock().unwrap();
        manager
            .broadcast(format!(
                "{{\"event\":\"pageupdate\",\"data\":{{\"page\":\"{}\"}}}}",
                path
            ))
            .await;
    }

    async fn on_dir_update(self: Arc<Self>) {
        let mut manager = self.websocket_manager.lock().unwrap();

        let legend = self.clone().get_legend().unwrap_or_default();
        let legend = BASE64_STANDARD.encode(
            serde_json::to_string(&legend)
                .map_err(|e| format!("Failed to serialize legend: {}", e))
                .unwrap_or_default(),
        );

        manager
            .broadcast(format!(
                "{{\"event\":\"legendupdate\",\"data\":{{\"legend\":\"{}\"}}}}",
                legend
            ))
            .await;
    }

    async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
        let mut manager = self.websocket_manager.lock().unwrap();
        manager.add_client(socket);
    }

    fn get_legend(self: Arc<Self>) -> Result<Vec<String>, String> {
        let glob_path = self.base_path.to_string_lossy().to_string();
        let pattern = glob_path.clone() + "/**/*.md";

        let mut legend = Vec::new();
        for entry in glob_with(&pattern, MatchOptions::new()).map_err(|e| e.to_string())? {
            if entry.is_err() {
                continue;
            }

            let path = entry.map_err(|e| e.to_string())?;
            let path = path.to_string_lossy().to_string();
            let path = path.trim_start_matches("./").to_string();
            let path = path.trim_start_matches(glob_path.as_str()).to_string();
            legend.push(path);
        }

        Ok(legend)
    }

    async fn server(self: Arc<Self>) -> Router {
        Router::new()
            .route(
                "/",
                get({
                    let server = Arc::clone(&self);
                    move || {
                        let server = Arc::clone(&server);
                        async move { server.root_handler() }
                    }
                }),
            )
            .route(
                "/ws",
                any({
                    let server = Arc::clone(&self);
                    move |ws: WebSocketUpgrade| {
                        let server = Arc::clone(&server);
                        async move { ws.on_upgrade(|socket| server.handle_socket(socket)) }
                    }
                }),
            )
            .fallback(get({
                let server = Arc::clone(&self);
                move |uri: Uri| {
                    let server = Arc::clone(&server);
                    async move {
                        server.build_content(
                            uri.path().to_string().trim_start_matches('/').to_string(),
                        )
                    }
                }
            }))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
    }

    pub async fn start(self: Arc<Self>) {
        let addr = SocketAddr::from((
            IpAddr::V4(Ipv4Addr::from_str(&self.host).expect("Failed to parse host")),
            self.port,
        ));

        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        tracing::info!("Webserver listening on {}", addr);

        let server = self.clone().server().await;
        tokio::spawn(async move {
            axum::serve(
                listener,
                server.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .expect("Failed to start server");
        });

        if !self.autoreload {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        let (mut watcher, mut rx) = async_watcher().expect("Failed to create watcher");
        watcher
            .watch(self.base_path.as_path(), RecursiveMode::Recursive)
            .expect("Failed to watch");

        while let Some(res) = rx.next().await {
            match res {
                Ok(event) => match event.kind {
                    EventKind::Modify(_) => {
                        let path = event.paths[0].to_string_lossy().to_string();
                        let path = path
                            .strip_prefix(&self.base_path.to_string_lossy().to_string())
                            .unwrap()
                            .to_string();

                        tracing::info!("\"{}\" was updated! Refreshing on all clients.", path);

                        self.clone().on_file_update(path).await;
                    }

                    EventKind::Create(_) => {
                        self.clone().on_dir_update().await;
                    }

                    EventKind::Remove(_) => {
                        self.clone().on_dir_update().await;
                    }

                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}
