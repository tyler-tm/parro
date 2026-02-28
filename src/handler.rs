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

        match Command::parse(&line) {
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
            Ok(Command::Delete(key)) => {
                let mut db = db.write().await;
                match db.delete(key) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::new_db;
    use tokio::io::AsyncBufReadExt;
    use tokio::net::TcpListener;

    async fn setup_test() -> (TcpStream, Db, String) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let db = new_db();

        let db_clone = db.clone();
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            process(socket, db_clone).await.unwrap();
        });

        let stream = TcpStream::connect(addr).await.unwrap();
        (stream, db, addr.to_string())
    }

    #[tokio::test]
    async fn process_get_found() {
        let (mut stream, db, _) = setup_test().await;
        db.write().await.set("key", "value").unwrap();

        stream.write_all(b"GET key\n").await.unwrap();
        stream.flush().await.unwrap();

        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "value\n");
    }

    #[tokio::test]
    async fn process_get_not_found() {
        let (mut stream, _, _) = setup_test().await;

        stream.write_all(b"GET key\n").await.unwrap();
        stream.flush().await.unwrap();

        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "NULL\n");
    }

    #[tokio::test]
    async fn process_set_success() {
        let (mut stream, db, _) = setup_test().await;

        stream.write_all(b"SET key value\n").await.unwrap();
        stream.flush().await.unwrap();

        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "OK\n");

        assert_eq!(db.read().await.get("key"), Some("value"));
    }

    #[tokio::test]
    async fn process_invalid_command() {
        let (mut stream, _, _) = setup_test().await;

        stream.write_all(b"INVALID command\n").await.unwrap();
        stream.flush().await.unwrap();

        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "Error: unknown command\n");
    }
}
