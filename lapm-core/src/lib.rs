use serde::{Serialize,Deserialize};

pub mod stream;

pub use stream::{
    IpcMessage,
    IpcError,
};


#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonCommand {
    Layer(DaemonLayerCommand),
    Entry(DaemonEntryCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonLayerCommand {
    Add{ name: String, password: String },
    List,
    Open{ name: String, password: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonEntryCommand {
    Add {
        name: String,
        password: String,
        layer: String,
    },
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
