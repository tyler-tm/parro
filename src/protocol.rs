use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::static_utils::BYTES_MB_CONVERSION;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Value(Vec<u8>),
    Null,
    Ok,
    Error(String),
}

const MAX_FRAME_SIZE: u32 = (64 * BYTES_MB_CONVERSION) as u32;

pub async fn write_frame(
    writer: &mut (impl AsyncWriteExt + Unpin),
    data: &[u8],
) -> std::io::Result<()> {
    let len = data.len() as u32;
    writer.write_u32(len).await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_frame(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> std::io::Result<Option<Vec<u8>>> {
    let len = match reader.read_u32().await {
        Ok(len) => len,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };

    if len > MAX_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame size {len} exceeds maximum {MAX_FRAME_SIZE}"),
        ));
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    Ok(Some(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let req = Request::Set {
            key: "k".into(),
            value: vec![1, 2, 3],
        };
        let bytes = bincode::serialize(&req).unwrap();
        let decoded: Request = bincode::deserialize(&bytes).unwrap();
        match decoded {
            Request::Set { key, value } => {
                assert_eq!(key, "k");
                assert_eq!(value, vec![1, 2, 3]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn response_roundtrip() {
        let resp = Response::Value(vec![42]);
        let bytes = bincode::serialize(&resp).unwrap();
        let decoded: Response = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded, Response::Value(vec![42]));
    }

    #[tokio::test]
    async fn frame_roundtrip() {
        let data = b"hello world";
        let mut buf = Vec::new();
        write_frame(&mut buf, data).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let result = read_frame(&mut cursor).await.unwrap();
        assert_eq!(result, Some(data.to_vec()));
    }

    #[tokio::test]
    async fn frame_eof_returns_none() {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let result = read_frame(&mut cursor).await.unwrap();
        assert_eq!(result, None);
    }
}
