use lapm_core::{
    command::layer::{
        DaemonLayerAddCommand, DaemonLayerConfigChangeCommand, DaemonLayerConfigShowCommand, DaemonLayerListCommand, DaemonLayerOpenCommand, LayerInfo
    },
    stream::{IpcCommand as _, StreamError},
};
use clap::Subcommand;
use tabled::{
    Table,
    Tabled,
    settings::{
        Color,
        Modify,
        Style,
        Width,
        object::{
            Columns,
            Rows,
        },
    },
};
use thiserror::Error;

use crate::password::{self, PasswordError};
use super::TimeoutArg;


#[derive(Debug,Error)]
pub enum LayerError {
    #[error("{0}")]
    PasswordError(#[from]PasswordError),
    #[error("{0}")]
    StreamError(#[from]StreamError),
}
type LayerResult<T> = Result<T,LayerError>;

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
pub enum LayerCommand {
    Add{
        name: String,
        #[command(flatten)]
        timeout: TimeoutArg,
    },
    List,
    Open{ name: String },
    Config{
        #[command(subcommand)]
        action: LayerConfigCommand,
    }
}

#[derive(Subcommand, Clone)]
pub enum LayerConfigCommand {
    Show{ layer: String },
    Change{
        layer: String,
        #[command(flatten)]
        timeout: Option<TimeoutArg>,
        #[arg(long)]
        public_usernames: Option<bool>,
    }
}

#[derive(Debug, Clone)]
pub struct LayerInfoTableRow(pub LayerInfo);
impl From<LayerInfo> for LayerInfoTableRow {
    fn from(value: LayerInfo) -> Self {
        Self(value)
    }
}

impl Tabled for LayerInfoTableRow {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        let info = &self.0;
        vec![
            info.name.as_str().into(),
            info.path.as_str().into(),
            (if info.open {"OPEN"} else {"CLOSED"}).into(),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            "Name".into(),
            "Path".into(),
            "Status".into(),
        ]
    }
}

impl FromIterator<LayerInfoTableRow> for Table {
    fn from_iter<T: IntoIterator<Item = LayerInfoTableRow>>(iter: T) -> Self {
        let vec = iter.into_iter().collect::<Vec<_>>();
        let mut table = Table::new(vec.clone());
        for (i, layer) in vec.into_iter().enumerate() {
            let color = if layer.0.open { Color::FG_GREEN } else { Color::FG_RED };
            table.with(Modify::new(Rows::one(i + 1)).with(color));
        }

        table
            .with(Style::modern())
            .with(Modify::new(Columns::last()).with(Width::increase(13)));
        table

    }
}

pub async fn handle_layer_command(command: LayerCommand) -> Result<(), LayerError> {
    match command {
        LayerCommand::Add{ name, timeout } => add_layer(name, timeout).await,
        LayerCommand::List => list_layers().await,
        LayerCommand::Open { name } => open_layer(name).await,
        LayerCommand::Config { action } => handle_layer_config_command(action).await,
    }
}

async fn add_layer(name: String, timeout: TimeoutArg) -> LayerResult<()> {
    let password = password::get_and_confirm_password()?;
    let timeout = if timeout.no_timeout {
        None
    } else {
        timeout.timeout
            .or(timeout.timeout_m.map(|tout| tout * 60))
            .or(timeout.timeout_h.map(|tout| tout * 3600))
    };

    // Open stream AFTER password is received
    let mut stream = lapm_core::stream::get_connection_stream().await?;
    DaemonLayerAddCommand{ name, password, timeout }
        .send(&mut stream).await?;
    Ok(())
}

async fn list_layers() -> LayerResult<()> {
    let mut stream = lapm_core::stream::get_connection_stream().await?;
    let res = DaemonLayerListCommand{}
        .send(&mut stream).await?;
    let table = res.layers.into_iter()
        .map(|lyr| LayerInfoTableRow::from(lyr))
        .collect::<Table>();
    println!("{table}");

    Ok(())
}

async fn open_layer(name: String) -> LayerResult<()> {
    let password = password::prompt_password(
        format!("Please enter the password for layer \"{name}\":"),
        3
    )?;

    // Open stream AFTER password is received
    let mut stream = lapm_core::stream::get_connection_stream().await?;
    DaemonLayerOpenCommand{ name, password }
        .send(&mut stream).await?;

    Ok(())
}

async fn handle_layer_config_command(command: LayerConfigCommand) -> LayerResult<()> {
    match command {
        LayerConfigCommand::Show{ layer } => show_layer_config(layer).await,
        LayerConfigCommand::Change { layer, timeout, public_usernames } => change_layer_config(layer, timeout, public_usernames).await,
    }
}

async fn show_layer_config(layer: String) -> LayerResult<()> {
    let mut stream = lapm_core::stream::get_connection_stream().await?;
    let res = DaemonLayerConfigShowCommand{ layer }
        .send(&mut stream).await?;
    println!("{res:?}");
    Ok(())
}

async fn change_layer_config(layer: String, timeout: Option<TimeoutArg>, public_usernames: Option<bool>) -> LayerResult<()> {
    let mut stream = lapm_core::stream::get_connection_stream().await?;

    let timeout = if let Some(tout) = timeout {
        if tout.no_timeout {
            Some(None)
        } else {
            Some(tout.timeout
                .or(tout.timeout_m.map(|tout| tout * 60))
                .or(tout.timeout_h.map(|tout| tout * 3600)))
        }
    } else { None };

    DaemonLayerConfigChangeCommand{ layer, timeout, public_usernames }
        .send(&mut stream).await?;
    Ok(())
}
