use parro::handler::process;
use parro::storage::{Db, new_db};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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
async fn test_process_get_found() {
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
async fn test_process_get_not_found() {
    let (mut stream, _, _) = setup_test().await;

    stream.write_all(b"GET key\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, "NULL\n");
}

#[tokio::test]
async fn test_process_set_success() {
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
async fn test_process_invalid_command() {
    let (mut stream, _, _) = setup_test().await;

    stream.write_all(b"INVALID command\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, "Error: unknown command\n");
}
