use crate::error::Result;
use crate::protocol::{self, Request, Response};
use crate::storage::Db;
use tokio::net::TcpStream;

pub async fn process(mut socket: TcpStream, db: Db) -> Result<()> {
    let (mut reader, mut writer) = socket.split();

    loop {
        let frame = protocol::read_frame(&mut reader).await?;
        let Some(frame) = frame else { break };

        let response = match bincode::deserialize::<Request>(&frame) {
            Ok(Request::Get { key }) => {
                let db = db.read().await;
                match db.get(&key) {
                    Some(value) => Response::Value(value.to_vec()),
                    None => Response::Null,
                }
            }
            Ok(Request::Set { key, value }) => {
                match db.write().await.set(&key, value) {
                    Ok(()) => Response::Ok,
                    Err(e) => Response::Error(e.to_string()),
                }
            }
            Ok(Request::Delete { key }) => {
                let _ = db.write().await.delete(&key);
                Response::Ok
            }
            Err(e) => Response::Error(format!("invalid request: {e}")),
        };

        let response_bytes = bincode::serialize(&response)?;
        protocol::write_frame(&mut writer, &response_bytes).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::new_db;
    use tokio::net::TcpListener;

    async fn setup_test() -> (TcpStream, Db) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let db = new_db();

        let db_clone = db.clone();
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            process(socket, db_clone).await.unwrap();
        });

        let stream = TcpStream::connect(addr).await.unwrap();
        (stream, db)
    }

    async fn send_request(stream: &mut TcpStream, request: &Request) -> Response {
        let (mut reader, mut writer) = stream.split();
        let request_bytes = bincode::serialize(request).unwrap();
        protocol::write_frame(&mut writer, &request_bytes).await.unwrap();
        let frame = protocol::read_frame(&mut reader).await.unwrap().unwrap();
        bincode::deserialize(&frame).unwrap()
    }

    #[tokio::test]
    async fn process_get_found() {
        let (mut stream, db) = setup_test().await;
        db.write().await.set("key", b"value".to_vec()).unwrap();

        let response = send_request(&mut stream, &Request::Get { key: "key".into() }).await;
        assert_eq!(response, Response::Value(b"value".to_vec()));
    }

    #[tokio::test]
    async fn process_get_not_found() {
        let (mut stream, _db) = setup_test().await;

        let response = send_request(&mut stream, &Request::Get { key: "key".into() }).await;
        assert_eq!(response, Response::Null);
    }

    #[tokio::test]
    async fn process_set_success() {
        let (mut stream, db) = setup_test().await;

        let response = send_request(
            &mut stream,
            &Request::Set { key: "key".into(), value: b"value".to_vec() },
        ).await;
        assert_eq!(response, Response::Ok);
        assert_eq!(db.read().await.get("key"), Some(b"value".as_slice()));
    }

    #[tokio::test]
    async fn process_delete_success() {
        let (mut stream, db) = setup_test().await;
        db.write().await.set("key", b"value".to_vec()).unwrap();

        let response = send_request(&mut stream, &Request::Delete { key: "key".into() }).await;
        assert_eq!(response, Response::Ok);
        assert_eq!(db.read().await.get("key"), None);
    }

    #[tokio::test]
    async fn process_delete_idempotent() {
        let (mut stream, _db) = setup_test().await;

        let response = send_request(&mut stream, &Request::Delete { key: "key".into() }).await;
        assert_eq!(response, Response::Ok);
    }
}
