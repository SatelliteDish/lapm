use serde::{Serialize,Deserialize};
use interprocess::local_socket::{
    Name,
    GenericNamespaced,
    tokio::{prelude::*,Stream},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum StreamError {
    #[error("{0}")]
    SerializationError(String),
    #[error("{0}")]
    DeserializationError(String),
    #[error("Failed to write to stream: {0}")]
    WriteError(String),
    #[error("Failed to read from stream: {0}")]
    ReadError(String),
}

#[derive(Debug, Serialize, Deserialize, Error)]
pub enum IpcError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Operation Failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage<T>(pub Result<T, IpcError>);

impl<T> From<Result<T,IpcError>> for IpcMessage<T>
where T: Serialize + for<'de> Deserialize<'de> {
    fn from(value: Result<T, IpcError>) -> Self {
        Self(value)
    }
}

impl<T> IpcMessage<T>
where T: Serialize + for<'de> Deserialize<'de> {
    pub fn from_ok(val: T) -> Self {
        Self(Ok(val))
    }

    pub fn from_err(err: IpcError) -> Self {
        Self(Err(err))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, StreamError> {
        postcard::to_allocvec(&self)
            .map_err(|e| StreamError::SerializationError(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StreamError> {
        postcard::from_bytes(bytes)
            .map_err(|e| StreamError::DeserializationError(e.to_string()))
    }

    pub async fn send(&self, stream: &mut Stream) -> Result<(), StreamError> {
        let bytes = self.to_bytes()?;
        stream.write_u32_le(bytes.len() as u32).await
            .map_err(|e| StreamError::WriteError(e.to_string()))?;
        stream.write_all(&bytes).await
            .map_err(|e| StreamError::WriteError(e.to_string()))
    }

    pub async fn receive(stream: &mut Stream) -> Result<T, StreamError> {
        let len = stream.read_u32_le().await
            .map_err(|e| StreamError::ReadError(e.to_string()))? as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await
            .map_err(|e| StreamError::ReadError(e.to_string()))?;
        Self::from_bytes(&buf)?.0
            .map_err(|e| StreamError::DeserializationError(e.to_string()))
    }

    pub async fn request<R: Serialize + for<'de> Deserialize<'de>>(&self, stream: &mut Stream) -> Result<R, StreamError> {
        self.send(stream).await?;
        IpcMessage::<R>::receive(stream).await
    }

    pub async fn request_empty(&self, stream: &mut Stream) -> Result<(), StreamError> {
        self.request::<()>(stream).await
    }
}


pub fn get_connection_name() -> Result<Name<'static>, String> {
    "lapm.sock".to_ns_name::<GenericNamespaced>()
        .map_err(|e| format!("Failed to create socket name: {e}"))
}

pub async fn get_connection_stream() -> Result<Stream, String> {
    let name = get_connection_name()?;
    Stream::connect(name).await
        .map_err(|e| e.to_string())
}
