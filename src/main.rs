mod command;
mod error;
mod handler;
mod server;
mod storage;

use server::Server;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    let server = Server::new().await?;
    server.run().await?;
    Ok(())
}
