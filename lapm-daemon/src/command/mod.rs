use interprocess::local_socket::{
    tokio::{
        prelude::*,
        Stream,
    },
    ListenerOptions,
};
use lapm_core::{
    command::DaemonCommand,
    stream::{
        self,
        StreamError,
    },
};
use thiserror::Error;
use std::sync::{Arc,Mutex};

use crate::{
    App, layer::LayerError,
};

mod entry;
use entry::execute_entry_command;
mod layer;
use layer::execute_layer_command;


#[derive(Debug,Error)]
pub enum CommandError {
    #[error("State failed to unlock")]
    MutexError,
    #[error("{0}")]
    StreamError(#[from]StreamError),
    #[error("{0}")]
    LayerError(#[from]LayerError),
    #[error("Cannot connect to socket")]
    SocketError,
}

pub async fn handle_commands(app: Arc<Mutex<App>>) -> Result<(), CommandError> {
    let sock_name = lapm_core::stream::get_connection_name()
        .map_err(|_| CommandError::SocketError)?;
    let listener = ListenerOptions::new()
        .name(sock_name.clone())
        .create_tokio()
        .map_err(|_| CommandError::SocketError)?;

    let local = tokio::task::LocalSet::new();

    local.run_until(async move {
        loop {
            let mut stream = listener.accept().await.unwrap();
            let app = app.clone();
            tokio::task::spawn_local(async move {
                match stream::receive::<DaemonCommand>(&mut stream).await {
                    Ok(command) => {
                        if let Err(e) = execute_command(app, command, &mut stream).await {
                            eprintln!("{e}");
                        }
                    },
                    Err(e) => eprintln!("{e}"),
                }
            });
        }
    }).await;
    Ok(())
}

async fn execute_command(app: Arc<Mutex<App>>, command: DaemonCommand, stream: &mut Stream) -> Result<(), CommandError> {
    match command {
        DaemonCommand::Layer(layer) => execute_layer_command(app, layer, stream).await,
        DaemonCommand::Entry(entry) => execute_entry_command(app, entry, stream).await,
    }
}
