use crate::error::Result;
use crate::handler;
use crate::static_utils;
use crate::storage;
use tokio::net::TcpListener;

// Server could likely be enumerated as TCP, gRPC, etc. in the future
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let addr = static_utils::default_addr();
        let listener = TcpListener::bind(&addr).await?;
        println!("🦜 Parro open for business at {addr}");
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
