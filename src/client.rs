use crate::error::Result;
use crate::protocol::{self, Request, Response};
use crate::static_utils;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;
use tokio::net::TcpStream;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Client { stream })
    }

    pub async fn connect_default() -> Result<Self> {
        Self::connect(&static_utils::default_addr()).await
    }

    pub async fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>> {
        let response = self
            .send(Request::Get {
                key: key.to_string(),
            })
            .await?;

        match response {
            Response::Value(bytes) => {
                let value: T = bincode::deserialize(&bytes)?;
                Ok(Some(value))
            }
            Response::Null => Ok(None),
            Response::Error(msg) => Err(msg.into()),
            Response::Ok => Err("unexpected OK response for GET".into()),
        }
    }

    pub async fn get_or_set<T, F, Fut>(&mut self, key: &str, compute: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        if let Some(value) = self.get::<T>(key).await? {
            return Ok(value);
        }
        let value = compute().await?;
        self.set(key, &value).await?;
        Ok(value)
    }

    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let value_bytes = bincode::serialize(value)?;
        let response = self
            .send(Request::Set {
                key: key.to_string(),
                value: value_bytes,
            })
            .await?;

        match response {
            Response::Ok => Ok(()),
            Response::Error(msg) => Err(msg.into()),
            _ => Err("unexpected response for SET".into()),
        }
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {
        let response = self
            .send(Request::Delete {
                key: key.to_string(),
            })
            .await?;

        match response {
            Response::Ok => Ok(()),
            Response::Error(msg) => Err(msg.into()),
            _ => Err("unexpected response for DELETE".into()),
        }
    }

    async fn send(&mut self, request: Request) -> Result<Response> {
        let request_bytes = bincode::serialize(&request)?;
        let (mut reader, mut writer) = self.stream.split();
        protocol::write_frame(&mut writer, &request_bytes).await?;
        let frame = protocol::read_frame(&mut reader)
            .await?
            .ok_or("connection closed")?;
        let response: Response = bincode::deserialize(&frame)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler;
    use crate::storage::new_db;
    use serde::{Deserialize, Serialize};
    use tokio::net::TcpListener;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct User {
        name: String,
        age: u32,
    }

    async fn setup_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let db = new_db();

        tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let db = db.clone();
                tokio::spawn(async move {
                    handler::process(socket, db).await.unwrap();
                });
            }
        });

        addr
    }

    #[tokio::test]
    async fn client_set_and_get_struct() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        let user = User {
            name: "Alice".into(),
            age: 30,
        };
        client.set("user:1", &user).await.unwrap();

        let retrieved: Option<User> = client.get("user:1").await.unwrap();
        assert_eq!(retrieved, Some(user));
    }

    #[tokio::test]
    async fn client_get_missing_key() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        let result: Option<String> = client.get("nonexistent").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn client_set_and_delete() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        client.set("key", &42u64).await.unwrap();
        client.delete("key").await.unwrap();

        let result: Option<u64> = client.get("key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn client_delete_missing_key() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        client.delete("nonexistent").await.unwrap();
    }

    #[tokio::test]
    async fn client_set_and_get_string() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        client
            .set("greeting", &"hello world".to_string())
            .await
            .unwrap();

        let result: Option<String> = client.get("greeting").await.unwrap();
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[tokio::test]
    async fn client_get_or_set_miss() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        let value: User = client
            .get_or_set("user:2", || async {
                Ok(User {
                    name: "Bob".into(),
                    age: 25,
                })
            })
            .await
            .unwrap();

        assert_eq!(
            value,
            User {
                name: "Bob".into(),
                age: 25
            }
        );

        // Verify it was stored
        let cached: Option<User> = client.get("user:2").await.unwrap();
        assert_eq!(cached, Some(value));
    }

    #[tokio::test]
    async fn client_get_or_set_hit() {
        let addr = setup_server().await;
        let mut client = Client::connect(&addr).await.unwrap();

        let user = User {
            name: "Alice".into(),
            age: 30,
        };
        client.set("user:1", &user).await.unwrap();

        // Should return cached value, not call compute
        let value: User = client
            .get_or_set("user:1", || async {
                panic!("should not be called");
            })
            .await
            .unwrap();

        assert_eq!(value, user);
    }
}
