use serde::{Serialize,Deserialize};

pub mod stream;

pub use stream::{
    IpcMessage,
    IpcError,
};


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
