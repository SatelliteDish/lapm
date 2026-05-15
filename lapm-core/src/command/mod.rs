use serde::{Serialize,Deserialize};
use derive_more::From;

pub mod layer;
pub mod entry;

use layer::DaemonLayerCommand;
use entry::DaemonEntryCommand;

#[derive(Debug, Serialize, Deserialize, From)]
pub enum DaemonCommand {
    Layer(#[from]DaemonLayerCommand),
    Entry(#[from]DaemonEntryCommand),
}
