use crate::command::Command;
use crate::error::Result;
use crate::storage::Db;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

async fn write_error(
    writer: &mut (impl AsyncWriteExt + Unpin),
    e: impl std::fmt::Display,
) -> Result<()> {
    let msg = format!("Error: {e}\n");
    writer.write_all(msg.as_bytes()).await?;
    Ok(())
}

async fn write_result(
    writer: &mut (impl AsyncWriteExt + Unpin),
    result: std::result::Result<(), impl std::fmt::Display>,
) -> Result<()> {
    match result {
        Ok(()) => writer.write_all(b"OK\n").await?,
        Err(e) => write_error(writer, e).await?,
    }
    Ok(())
}

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
                match db.get(key) {
                    Some(value) => {
                        writer.write_all(value.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                    }
                    None => writer.write_all(b"NULL\n").await?,
                }
            }
            Ok(Command::Set(key, value)) => {
                let result = db.write().await.set(key, value);
                write_result(&mut writer, result).await?;
            }
            Ok(Command::Delete(key)) => {
                let result = db.write().await.delete(key);
                write_result(&mut writer, result).await?;
            }
            Err(e) => write_error(&mut writer, e).await?,
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
