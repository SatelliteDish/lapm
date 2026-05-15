use interprocess::local_socket::{
    tokio::{
        prelude::*,
        Stream,
    },
    ListenerOptions,
};
use lapm_core::{
    IpcCommand as _, IpcError, command::{
        DaemonCommand,
        entry::{
            DaemonEntryAddCommand,
            DaemonEntryCommand,
            DaemonEntryListCommand,
        },
        layer::{
            DaemonLayerAddCommand,
            DaemonLayerCommand,
            DaemonLayerListCommand,
            DaemonLayerOpenCommand,
            LayerInfo,
            ListLayersResponse,
        },
    }, stream
};
use std::{sync::{Arc,Mutex}};

use crate::{
    App,
};

pub async fn handle_commands(app: Arc<Mutex<App>>) -> Result<(), String> {
    let sock_name = lapm_core::stream::get_connection_name()?;
    let listener = ListenerOptions::new().name(sock_name.clone()).create_tokio().map_err(|e| e.to_string())?;

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
        DaemonLayerCommand::Add(cmd) => {
            let DaemonLayerAddCommand { name, password } = cmd;
            let add_res = state.add_layer(name, password)
                .map_err(|e| IpcError::Unauthorized(e));
            DaemonLayerAddCommand::respond(add_res, stream).await
        },
        DaemonLayerCommand::List(_) => {
            let layer_info = state.config.layers.iter()
                .map(|lyr| LayerInfo::from(lyr.clone()))
                .collect::<Vec<_>>();
            DaemonLayerListCommand::respond_ok(
                ListLayersResponse{ layers: layer_info },
                stream
            ).await
        },
        DaemonLayerCommand::Open(cmd) => {
            let DaemonLayerOpenCommand{ name, password } = cmd;
            let layer_opt = state.config.layers.iter_mut()
                .find(|lyr| lyr.name.as_str() == name.as_str());

            if let Some(layer) = layer_opt {
                DaemonLayerOpenCommand::respond(
                    layer.open(&password)
                        .map_err(|e| IpcError::Unauthorized(e)),
                    stream,
                ).await
            } else {
                DaemonLayerOpenCommand::respond_err(
                    IpcError::NotFound(format!("layer {name} not found")),
                    stream,
                ).await
            }
        },
    }.map_err(|e| e.into())
}


async fn execute_entry_command(app: Arc<Mutex<App>>, command: DaemonEntryCommand, stream: &mut Stream) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = app.lock()
        .map_err(|e| e.to_string())?;

    match command {
        DaemonEntryCommand::Add(cmd)  => {
            let DaemonEntryAddCommand{ entry, layer } = cmd;
            let found = state.config.layers.iter_mut()
                .find(|lyr| lyr.name.as_str() == layer.as_str());
            if let Some(layer) = found {
                DaemonEntryAddCommand::respond(
                    layer.add_entry(entry.into())
                        .map_err(|e| {
                            eprint!("{e}");
                            IpcError::from(e)
                        }),
                    stream,
                ).await
            } else {
                DaemonEntryAddCommand::respond_err(
                    IpcError::NotFound(format!("Could not find Layer \"{layer}\"")),
                    stream,
                ).await
            }
        },
        DaemonEntryCommand::List(cmd) => {
            let DaemonEntryListCommand { url, copy } = cmd;
            let mut entries = state.config.layers
                .iter().filter_map(|lyr| {
                    lyr.get_entries().ok()
                }).flatten()
                .collect::<Vec<_>>();
            if let Some(url) = url {
                entries = entries.into_iter()
                    .filter(|ent| ent.url.as_deref() == Some(url.as_str()))
                    .collect::<Vec<_>>();
            }
            if copy {
                match entries.len() {
                    0 => return DaemonEntryListCommand::respond_err(
                        IpcError::NotFound("No matching entries found, nothing was copied".to_string()),
                        stream,
                    ).await.map_err(|e| e.into()),
                    1 => {
                        let password = entries[0].password.clone();
                        tokio::task::spawn_blocking(move || {
                            let mut clipboard = arboard::Clipboard::new()?;
                            clipboard.set_text(&password)?;
                            std::thread::sleep(std::time::Duration::from_secs(10));
                            if clipboard.get_text().ok().as_deref() == Some(&password) {
                                let _ = clipboard.set_text("");
                            }
                            Ok::<_, arboard::Error>(())
                        });
                    },
                    _ => return DaemonEntryListCommand::respond_err(
                        IpcError::BadRequest("More than one match for query. Nothing copied, please be more specific".to_string()),
                        stream,
                    ).await.map_err(|e| e.into()),
                }
            }
            DaemonEntryListCommand::respond_ok(entries, stream).await
        },
    }.map_err(|e| e.into())
}
