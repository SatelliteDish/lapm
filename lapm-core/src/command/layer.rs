use serde::{Serialize,Deserialize};
use derive_more::From;

use super::DaemonCommand;
use crate::IpcCommand;

#[derive(Debug, Serialize, Deserialize, From)]
pub enum DaemonLayerCommand {
    Add(#[from]DaemonLayerAddCommand),
    List(#[from]DaemonLayerListCommand),
    Open(#[from]DaemonLayerOpenCommand),
    Config(#[from]DaemonLayerConfigCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonLayerAddCommand {
    pub name: String,
    pub password: String,
    pub timeout: Option<u64>,
}

impl IpcCommand for DaemonLayerAddCommand {
    type Success = ();
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonLayerCommand::from(self).into()
    }
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

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct DaemonLayerListCommand {}

impl IpcCommand for DaemonLayerListCommand {
    type Success = ListLayersResponse;
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonLayerCommand::from(self).into()
    }
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct DaemonLayerOpenCommand {
    pub name: String,
    pub password: String,
}

impl IpcCommand for DaemonLayerOpenCommand {
    type Success = ();
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonLayerCommand::from(self).into()
    }
}


#[derive(Debug,Serialize,Deserialize,Clone,From)]
pub struct DaemonLayerConfig {
    pub timeout: Option<u64>,
    pub public_usernames: bool,
}

#[derive(Debug,Serialize,Deserialize,Clone,From)]
pub enum DaemonLayerConfigCommand {
    Show(#[from]DaemonLayerConfigShowCommand),
    Change(#[from]DaemonLayerConfigChangeCommand),
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct DaemonLayerConfigShowCommand {
    pub layer: String,
}

impl IpcCommand for DaemonLayerConfigShowCommand {
    type Success = DaemonLayerConfig;
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonLayerCommand::from(
            DaemonLayerConfigCommand::Show(self),
        ).into()
    }
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct DaemonLayerConfigChangeCommand {
    pub layer: String,
    pub timeout: Option<Option<u64>>,
    pub public_usernames: Option<bool>,
}

impl IpcCommand for DaemonLayerConfigChangeCommand {
    type Success = ();
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonLayerCommand::from(
            DaemonLayerConfigCommand::Change(self),
        ).into()
    }
}
