use serde::{Serialize,Deserialize};
use derive_more::From;

use super::DaemonCommand;
use crate::{
    IpcCommand,
    IpcError,
};

#[derive(Debug, Serialize, Deserialize, From)]
pub enum DaemonEntryCommand {
    Add(#[from]DaemonEntryAddCommand),
    List(#[from]DaemonEntryListCommand),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DaemonEntryAddCommand {
    pub name: String,
    pub password: String,
    pub layer: String,
}

impl IpcCommand for DaemonEntryAddCommand {
    type Success = ();

    fn into_envelope(self) -> DaemonCommand {
        DaemonEntryCommand::from(self).into()
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DaemonEntryListCommand {}

impl IpcCommand for DaemonEntryListCommand {
    type Success = ();

    fn into_envelope(self) -> DaemonCommand {
        DaemonEntryCommand::from(self).into()
    }
}
