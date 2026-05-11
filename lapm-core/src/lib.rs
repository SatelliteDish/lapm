use serde::{Serialize,Deserialize};
use interprocess::local_socket::{
    Name,
    GenericNamespaced,
    tokio::{prelude::*,Stream},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait IpcMessage: Serialize + for<'de> Deserialize<'de> {
    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&self)
            .map_err(|e| e.to_string())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        postcard::from_bytes(bytes)
            .map_err(|e| e.to_string())
    }

    async fn send(&self, stream: &mut Stream) -> Result<(), String> {
        let bytes = self.to_bytes()?;
        stream.write_u32_le(bytes.len() as u32).await
            .map_err(|e| e.to_string())?;
        stream.write_all(&bytes).await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn receive(stream: &mut Stream) -> Result<Self, String> {
        let len = stream.read_u32_le().await
            .map_err(|e| e.to_string())? as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await
            .map_err(|e| e.to_string())?;
        Self::from_bytes(&buf)
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
}

impl IpcMessage for LapmCommand {}

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

impl IpcMessage for ListLayersResponse {}

pub fn get_connection_name() -> Result<Name<'static>, String> {
    "lapm.sock".to_ns_name::<GenericNamespaced>()
        .map_err(|e| format!("Failed to create socket name: {e}"))
}
