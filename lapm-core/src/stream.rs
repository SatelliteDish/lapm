use serde::{Serialize,Deserialize};
use interprocess::local_socket::{
    Name,
    GenericNamespaced,
    tokio::{prelude::*,Stream},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use thiserror::Error;

use crate::command::DaemonCommand;


#[derive(Debug, Serialize, Deserialize, Error)]
pub enum IpcError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Operation Failed: {0}")]
    OperationFailed(String),
}

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
    #[error("{0}")]
    DaemonError(#[from]IpcError),
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




fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, StreamError> {
    postcard::to_allocvec(&value)
        .map_err(|e| StreamError::SerializationError(e.to_string()))
}

fn from_bytes<R: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<R, StreamError> {
    postcard::from_bytes(bytes)
        .map_err(|e| StreamError::DeserializationError(e.to_string()))
}

async fn write_to_stream(bytes: &[u8], stream: &mut Stream) -> Result<(), StreamError> {
    stream.write_u32_le(bytes.len() as u32).await
        .map_err(|e| StreamError::WriteError(e.to_string()))?;
    stream.write_all(&bytes).await
        .map_err(|e| StreamError::WriteError(e.to_string()))
}

pub async fn receive<R: for<'de> Deserialize<'de>>(stream: &mut Stream) -> Result<R, StreamError> {
    let len = stream.read_u32_le().await
        .map_err(|e| StreamError::ReadError(e.to_string()))? as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await
        .map_err(|e| StreamError::ReadError(e.to_string()))?;
    from_bytes::<R>(&buf)
}

pub trait IpcCommand: Sized + Serialize + for<'de> Deserialize<'de> {
    type Success: Sized + Serialize + for<'de> Deserialize<'de>;

    fn into_envelope(self) -> DaemonCommand;

    async fn send(self, stream: &mut Stream) -> Result<Self::Success, StreamError> {
        let bytes = to_bytes(&self.into_envelope())?;
        write_to_stream(&bytes, stream).await?; // Send this message
        receive::<Result<Self::Success,IpcError>>(stream).await? // Listen for fallible response
            .map_err(|e| e.into())
    }

    async fn respond(value: Result<Self::Success, IpcError>, stream: &mut Stream) -> Result<(), StreamError> {
        write_to_stream(&to_bytes(&value)?, stream).await
    }

    async fn respond_ok(value: Self::Success, stream: &mut Stream) -> Result<(),StreamError> {
        Self::respond(Ok(value), stream).await
    }

    async fn respond_err(err: IpcError, stream: &mut Stream) -> Result<(),StreamError> {
        Self::respond(Err(err), stream).await
    }
}
