use interprocess::local_socket::{
    tokio::{
        prelude::*,
        Stream,
    },
    ListenerOptions,
};
use lapm_core::{
    IpcError, IpcMessage, DaemonCommand, DaemonLayerCommand, LayerInfo, ListLayersResponse, DaemonEntryCommand,
};
use std::sync::{Arc,Mutex};

use crate::App;

pub async fn handle_commands(app: Arc<Mutex<App>>) -> Result<(), String> {
    let sock_name = lapm_core::stream::get_connection_name()?;
    let listener = ListenerOptions::new().name(sock_name.clone()).create_tokio().map_err(|e| e.to_string())?;

    let local = tokio::task::LocalSet::new();

    local.run_until(async move {
        loop {
            let mut stream = listener.accept().await.unwrap();
            let app = app.clone();
            tokio::task::spawn_local(async move {
                match IpcMessage::<DaemonCommand>::receive(&mut stream).await {
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

async fn execute_command(app: Arc<Mutex<App>>, command: DaemonCommand, stream: &mut Stream) -> Result<(),Box<dyn std::error::Error>> {
    match command {
        DaemonCommand::Layer(layer) => execute_layer_command(app, layer, stream).await,
        DaemonCommand::Entry(entry) => execute_entry_command(app, entry, stream).await,
    }.map_err(|e| e.into())
}

async fn execute_layer_command(app: Arc<Mutex<App>>, command: DaemonLayerCommand, stream: &mut Stream) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = app.lock()
        .map_err(|e| e.to_string())?;
    match command {
        DaemonLayerCommand::Add { name, password } => {
            let add_res = state.add_layer(name, password)
                .map_err(|e| IpcError::Unauthorized(e));
            IpcMessage::from(add_res).send(stream).await
        },
        DaemonLayerCommand::List => {
            let layer_info = state.config.layers.iter()
                .map(|lyr| LayerInfo::from(lyr.clone()))
                .collect::<Vec<_>>();
            IpcMessage::from_ok(
                ListLayersResponse{ layers: layer_info }
            )
                .send(stream).await
        },
        DaemonLayerCommand::Open { name, password } => {
            let layer_opt = state.config.layers.iter_mut()
                .find(|lyr| lyr.name.as_str() == name.as_str());

            if let Some(layer) = layer_opt {
                IpcMessage(
                    layer.open(&password)
                        .map_err(|e| IpcError::Unauthorized(e))
                )
                    .send(stream).await
                    .map_err(|e| e.into())
            } else {
                IpcMessage::<()>::from_err(
                    IpcError::NotFound(
                        format!("layer {name} not found")
                    )
                )
                    .send(stream).await
            }
        },
    }.map_err(|e| e.into())
}


async fn execute_entry_command(app: Arc<Mutex<App>>, command: DaemonEntryCommand, stream: &mut Stream) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = app.lock()
        .map_err(|e| e.to_string())?;

    match command {
        DaemonEntryCommand::Add { name, password, layer } => {
            if let Some(lyr) = state.config.layers.iter().find(|lyr| lyr.name.as_str() == layer.as_str()) {

                todo!()
            } else {
                todo!()
            }
        }
    }
}
