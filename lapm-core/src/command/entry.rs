use serde::{Serialize,Deserialize};
use derive_more::From;

use super::DaemonCommand;
use crate::IpcCommand;

#[derive(Debug, Serialize, Deserialize, From)]
pub enum DaemonEntryCommand {
    Add(#[from]DaemonEntryAddCommand),
    List(#[from]DaemonEntryListCommand),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DaemonEntryAddCommand{
    pub layer: String,
    pub entry: DaemonEntry,
}

impl IpcCommand for DaemonEntryAddCommand {
    type Success = ();
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonEntryCommand::from(self).into()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonEntry {
    pub title: Option<String>,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

impl DaemonEntry {
    pub fn new(username: String, password: String) -> Self {
        Self {
            title: None,
            username,
            password,
            url: None,
            notes: None,
        }
    }
}

pub type DaemonEntryListResponse = Vec<DaemonEntry>;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DaemonEntryListCommand {
    pub url: Option<String>,
    pub copy: bool,
}

impl IpcCommand for DaemonEntryListCommand {
    type Success = DaemonEntryListResponse;
    type Envelope = DaemonCommand;

    fn into_envelope(self) -> Self::Envelope {
        DaemonEntryCommand::from(self).into()
    }
}
