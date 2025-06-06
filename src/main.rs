pub mod server;
pub mod watcher;
pub mod websockets;

use crate::server::DocsmanServer;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
pub struct DocsmanArguments {
    pub path: PathBuf,
    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,
    #[clap(short, long, default_value = "0.0.0.0")]
    pub host: String,
    #[clap(short, long)]
    pub autoreload: Option<bool>,
    #[clap(short, long)]
    pub legend: Option<bool>,
}

#[tokio::main]
async fn main() {
    let arguments = DocsmanArguments::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let server = Arc::new(DocsmanServer::new(arguments));
    server.start().await;
}
