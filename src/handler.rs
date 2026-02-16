use crate::command::Command;
use crate::error::Result;
use crate::storage::Db;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

pub async fn process(mut socket: TcpStream, db: Db) -> Result<()> {
    let (reader, writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        match Command::from_str(&line) {
            Ok(Command::Get(key)) => {
                let db = db.read().await;
                if let Some(value) = db.get(key) {
                    writer.write_all(value.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                } else {
                    writer.write_all(b"NULL\n").await?;
                }
            }
            Ok(Command::Set(key, value)) => {
                let mut db = db.write().await;
                match db.set(key, value) {
                    Ok(_) => {
                        writer.write_all(b"OK\n").await?;
                    }
                    Err(e) => {
                        let msg = format!("Error: {}\n", e);
                        writer.write_all(msg.as_bytes()).await?;
                    }
                }
            }
            Err(e) => {
                let msg = format!("Error: {}\n", e);
                writer.write_all(msg.as_bytes()).await?;
            }
        }
        writer.flush().await?;
    }
    Ok(())
}
