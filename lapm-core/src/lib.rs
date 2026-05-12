use serde::{Serialize,Deserialize};
use interprocess::local_socket::{
    Name,
    GenericNamespaced,
    tokio::{prelude::*,Stream},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use thiserror::Error;


#[derive(Debug, Serialize, Deserialize, Error)]
pub enum IpcError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage<T>(pub Result<T, IpcError>);

impl<T> From<T> for IpcMessage<T>
where T: Serialize + for<'de> Deserialize<'de> {
    fn from(value: T) -> Self {
        Self(Ok(value))
    }
}

impl<T> IpcMessage<T>
where T: Serialize + for<'de> Deserialize<'de> {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&self)
            .map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        postcard::from_bytes(bytes)
            .map_err(|e| e.to_string())
    }

    pub async fn send(&self, stream: &mut Stream) -> Result<(), String> {
        let bytes = self.to_bytes()?;
        stream.write_u32_le(bytes.len() as u32).await
            .map_err(|e| e.to_string())?;
        stream.write_all(&bytes).await
            .map_err(|e| e.to_string())
    }

    pub async fn receive(stream: &mut Stream) -> Result<T, String> {
        let len = stream.read_u32_le().await
            .map_err(|e| e.to_string())? as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await
            .map_err(|e| e.to_string())?;
        Self::from_bytes(&buf)?.0
            .map_err(|e| e.to_string())
    }

    pub async fn request<R: Serialize + for<'de> Deserialize<'de>>(&self, stream: &mut Stream) -> Result<R, String> {
        self.send(stream).await?;
        IpcMessage::<R>::receive(stream).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LapmCommand {
    Layer(LapmLayerCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LapmLayerCommand {
    Add{ name: String, password: String },
    List,
    Open{ name: String, password: String },
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct LayerInfo {
    pub name: String,
    pub path: String,
    pub open: bool,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct ListLayersResponse {
    pub layers: Vec<LayerInfo>,
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
