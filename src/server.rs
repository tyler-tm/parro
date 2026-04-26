use crate::config::Config;
use crate::error::Result;
use crate::handler;
use crate::static_utils::BYTES_MB_CONVERSION;
use crate::storage::{self, Db};
use tokio::net::TcpListener;

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
        loop {
            let (socket, _) = self.listener.accept().await?;
            let db = self.db.clone();

            tokio::spawn(async move {
                if let Err(e) = handler::process(socket, db).await {
                    eprintln!("Error processing connection: {}", e);
                }
            });
        }
    }
}
