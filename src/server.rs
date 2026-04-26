use crate::config::Config;
use crate::error::Result;
use crate::handler;
use crate::static_utils::BYTES_MB_CONVERSION;
use crate::storage::{self, Db};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinSet;

pub struct Server {
    listener: TcpListener,
    db: Db,
}

impl Server {
    pub async fn new(config: Config) -> Result<Self> {
        let listener = TcpListener::bind(&config.addr).await?;
        let db = storage::new_db(config.max_size_bytes);
        println!(
            "🦜 Parro open for business at {} (max size: {} MB)",
            config.addr,
            config.max_size_bytes / BYTES_MB_CONVERSION
        );
        Ok(Server { listener, db })
    }

    pub async fn run(self) -> Result<()> {
        let (shutdown_tx, _) = watch::channel(false);
        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                biased;
                _ = tokio::signal::ctrl_c() => {
                    println!("Received shutdown signal; emptying connections...");
                    let _ = shutdown_tx.send(true);
                    break;
                }
                accept = self.listener.accept() => {
                    let (socket, _) = accept?;
                    let db = self.db.clone();
                    let shutdown_rx = shutdown_tx.subscribe();
                    tasks.spawn(async move {
                        if let Err(e) = handler::process(socket, db, shutdown_rx).await {
                            eprintln!("Error processing connection: {}", e);
                        }
                    });
                }
            }
        }

        while tasks.join_next().await.is_some() {}
        println!("Shutdown complete, cya.");
        Ok(())
    }
}
