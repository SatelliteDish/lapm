use serde::{Serialize,Deserialize};
use derive_more::From;

pub mod stream;
pub use stream::{
    IpcError,
    IpcCommand,
};

pub mod command;
