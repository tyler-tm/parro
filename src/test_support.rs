use crate::handler;
use crate::storage::{Db, new_db};
use tokio::net::TcpListener;
use tokio::sync::watch;

pub async fn start_server(max_size_bytes: usize) -> (String, Db, watch::Sender<bool>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let db = new_db(max_size_bytes);
    let (shutdown_tx, _) = watch::channel(false);

    let db_inner = db.clone();
    let shutdown_tx_inner = shutdown_tx.clone();
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let db = db_inner.clone();
            let shutdown_rx = shutdown_tx_inner.subscribe();
            tokio::spawn(async move {
                handler::process(socket, db, shutdown_rx).await.unwrap();
            });
        }
    });

    (addr, db, shutdown_tx)
}
