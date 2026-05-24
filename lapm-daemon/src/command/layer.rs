use interprocess::local_socket::tokio::Stream;
use lapm_core::{
    IpcCommand as _,
    IpcError,
    command::layer::{
        DaemonLayerAddCommand,
        DaemonLayerCommand,
        DaemonLayerConfigChangeCommand,
        DaemonLayerConfigCommand,
        DaemonLayerConfigShowCommand,
        DaemonLayerListCommand,
        DaemonLayerOpenCommand,
        LayerInfo,
        ListLayersResponse,
    },
};
use std::time::Duration;

use crate::{
    AppState,
    layer::config::LayerConfig,
};
use super::CommandError;


pub async fn execute_layer_command(app: AppState, command: DaemonLayerCommand, stream: &mut Stream) -> Result<(), CommandError> {
    match command {
        DaemonLayerCommand::Add(cmd) => add_layer(app, stream, cmd).await,
        DaemonLayerCommand::List(cmd) => list_layers(app, stream, cmd).await,
        DaemonLayerCommand::Open(cmd) => open_layer(app, stream, cmd).await,
        DaemonLayerCommand::Config(cmd) => execute_layer_config_command(app, cmd, stream).await,
    }
}

async fn add_layer(app: AppState, stream: &mut Stream, cmd: DaemonLayerAddCommand) -> Result<(),CommandError> {
    let mut state = app.lock()
        .map_err(|_| CommandError::MutexError)?;
    let DaemonLayerAddCommand { name, password, timeout } = cmd;
    let add_res = state.add_layer(name, password, timeout).await
        .map_err(|e| IpcError::Unauthorized(e));
    DaemonLayerAddCommand::respond(add_res, stream).await
        .map_err(|e| e.into())
}

async fn list_layers(app: AppState, stream: &mut Stream, _: DaemonLayerListCommand) -> Result<(), CommandError> {
            let state = app.lock()
                .map_err(|_| CommandError::MutexError)?;
            let layer_info = state.config.layers.iter()
                .map(|lyr| LayerInfo::from(lyr.clone()))
                .collect::<Vec<_>>();
            DaemonLayerListCommand::respond_ok(
                ListLayersResponse{ layers: layer_info },
                stream
            ).await
                .map_err(|e| e.into())
        }

async fn open_layer(app: AppState, stream: &mut Stream, cmd: DaemonLayerOpenCommand) -> Result<(),CommandError> {
            let mut state = app.lock()
                .map_err(|_| CommandError::MutexError)?;
            let DaemonLayerOpenCommand{ name, password } = cmd;
            let layer_opt = state.config.layers.iter_mut()
                .find(|lyr| lyr.name.as_str() == name.as_str());

            if let Some(layer) = layer_opt {
                DaemonLayerOpenCommand::respond(
                    layer.open(&password)
                        .map_err(|e| IpcError::Unauthorized(e.to_string())),
                    stream,
                ).await
                    .map_err(|e| e.into())
            } else {
                DaemonLayerOpenCommand::respond_err(
                    IpcError::NotFound(format!("layer {name} not found")),
                    stream,
                ).await
                    .map_err(|e| e.into())
            }
        }

async fn execute_layer_config_command(app: AppState, command: DaemonLayerConfigCommand, stream: &mut Stream) -> Result<(), CommandError> {
    match command {
        DaemonLayerConfigCommand::Show(cmd) => show_layer_config(app, stream, cmd).await,
        DaemonLayerConfigCommand::Change(cmd) => change_layer_config(app, stream, cmd).await,
    }

}

async fn show_layer_config(app: AppState, stream: &mut Stream, cmd: DaemonLayerConfigShowCommand) -> Result<(), CommandError> {
    let DaemonLayerConfigShowCommand { layer } = cmd;
    let state = app.lock()
        .map_err(|_| CommandError::MutexError)?;

    match state.config.layers.iter()
        .find(|lyr| lyr.name.as_str() == layer.as_str()) {
        Some(lyr) => {
            DaemonLayerConfigShowCommand::respond(
                lyr.get_config()
                    .map(|cfg| cfg.into())
                    .map_err(|e| IpcError::Unauthorized(e.to_string())),
                stream,
            ).await
        },
        None => {
            DaemonLayerConfigChangeCommand::respond(
                Err(IpcError::NotFound(format!("layer {layer} not found"))),
                stream,
            ).await
        },
    }.map_err(|e| e.into())
}

async fn change_layer_config(app: AppState, stream: &mut Stream, cmd: DaemonLayerConfigChangeCommand) -> Result<(), CommandError> {
    let DaemonLayerConfigChangeCommand { layer, timeout, public_usernames } = cmd;
    let mut state = app.lock()
        .map_err(|_| CommandError::MutexError)?;

    match state.config.layers.iter_mut()
        .find(|lyr| lyr.name.as_str() == layer.as_str()) {
        Some(lyr) => {
            let cfg = lyr.get_config()?;
            let res = lyr.set_config(LayerConfig {
                timeout: if let Some(tout) = timeout {
                    tout.map(|t| Duration::new(t,0))
                } else { cfg.timeout },
                public_usernames: if let Some(pub_users) = public_usernames {
                    pub_users
                } else { cfg.public_usernames },
            });
            DaemonLayerConfigChangeCommand::respond(
                res
                    .map(|_| ())
                    .map_err(|e| IpcError::Unauthorized(e.to_string())),
                stream,
            ).await
        },
        None => {
            DaemonLayerConfigShowCommand::respond(
                Err(IpcError::NotFound(format!("layer {layer} not found"))),
                stream,
            ).await
        }
    }.map_err(|e| e.into())
}
