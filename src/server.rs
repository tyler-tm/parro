use crate::error::Result;
use crate::handler;
use crate::storage;
use colored::*;
use std::env;
use tokio::net::TcpListener;

// Server could likely be enumerated as TCP, gRPC, etc. in the future
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let port = env::var("PARRO_PORT").unwrap_or_else(|_| "14242".to_string());
        let ip = "127.0.0.1";
        let addr = format!("{}:{}", ip, port);
        let listener = TcpListener::bind(&addr).await?;
        println!(
            "🦜 Parro open for business at {}:{}",
            ip.bright_blue(),
            port.bright_green()
        );
        Ok(Server { listener })
    }

    pub async fn run(self) -> Result<()> {
        let db = storage::new_db();

        loop {
            let (socket, _) = self.listener.accept().await?;
            let db = db.clone();

            tokio::spawn(async move {
                if let Err(e) = handler::process(socket, db).await {
                    eprintln!("Error processing connection: {}", e);
                }
            });
        }
    }
}
