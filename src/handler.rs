use crate::command::Command;
use crate::error::Result;
use crate::storage::Db;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn process(mut socket: TcpStream, db: Db) -> Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        match Command::from_str(&line) {
            Ok(Command::Get(key)) => {
                let db = db.lock().await;
                if let Some(value) = db.get(&key) {
                    writer.write_all(value.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                } else {
                    writer.write_all(b"NULL\n").await?;
                }
            }
            Ok(Command::Set(key, value)) => {
                let mut db = db.lock().await;
                db.insert(key, value);
                writer.write_all(b"OK\n").await?;
            }
            Err(e) => {
                let msg = format!("{}\n", e);
                writer.write_all(msg.as_bytes()).await?;
            }
        }
    }
    Ok(())
}
