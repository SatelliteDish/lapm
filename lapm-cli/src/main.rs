use lapm_core::{
    command::{
        entry::{
            DaemonEntry, DaemonEntryAddCommand, DaemonEntryListCommand
        }, layer::{
            DaemonLayerAddCommand,
            DaemonLayerConfigChangeCommand,
            DaemonLayerConfigShowCommand,
            DaemonLayerListCommand,
            DaemonLayerOpenCommand,
        }
    },
    stream::IpcCommand as _,
};
use clap::{Parser,Subcommand,Args};
use tabled::{
    Table,
};
mod password;
mod entry;
mod layer;

use entry::{
    EntryCommand,
    EntryTable,
    EntryTableRow,
};
use layer::{
    LayerConfigCommand,
    LayerCommand,
    LayerInfoTable,
    LayerInfoTableRow,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}


#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
enum CliCommand {
    Layer {
        #[command(subcommand)]
        layer: LayerCommand,
    },
    Entry {
        #[command(subcommand)]
        entry: EntryCommand,
    },
}

#[derive(Args, Clone)]
#[group(multiple = false)]
struct TimeoutArg {
    #[arg(long, value_name = "SECONDS")]
    timeout: Option<u64>,
    #[arg(long, value_name = "MINUTES")]
    timeout_m: Option<u64>,
    #[arg(long, value_name = "HOURS")]
    timeout_h: Option<u64>,
    #[arg(long)]
    no_timeout: bool,
}






#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        CliCommand::Layer{ layer } => {
            match layer {
                LayerCommand::Add{ name, timeout } => {
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
                },
                LayerCommand::List => {
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    let res = DaemonLayerListCommand{}
                        .send(&mut stream).await?;
                    let table = res.layers.into_iter()
                        .map(|lyr| LayerInfoTableRow::from(lyr))
                        .collect::<Table>();
                    println!("{table}");

                    Ok(())
                },
                LayerCommand::Open { name } => {
                    let password = password::prompt_password(
                        format!("Please enter the password for layer \"{name}\":"),
                        3
                    )?;

                    // Open stream AFTER password is received
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    DaemonLayerOpenCommand{ name, password }
                        .send(&mut stream).await?;

                    Ok(())
                },
                LayerCommand::Config { action } => {
                    match action {
                        LayerConfigCommand::Show{ layer } => {
                            let mut stream = lapm_core::stream::get_connection_stream().await?;
                            let res = DaemonLayerConfigShowCommand{ layer }
                                .send(&mut stream).await?;
                            println!("{res:?}");
                            Ok(())
                        },
                        LayerConfigCommand::Change { layer, timeout, public_usernames } => {
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
                    }
                }
            }
        },
        CliCommand::Entry { entry } => {
            match entry {
                EntryCommand::Add { name, layer, url } => {
                    let pwd = password::get_and_confirm_password()?;
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    DaemonEntryAddCommand {
                        layer,
                        entry: DaemonEntry {
                            username: name,
                            password: pwd,
                            url,
                            notes: None,
                            title: None,
                        },
                    }
                        .send(&mut stream).await?;
                    Ok(())
                },
                EntryCommand::List{ url, show, copy } => {
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    let entries = DaemonEntryListCommand{ url, copy }
                        .send(&mut stream).await?;

                    if !copy || show {
                        let table = EntryTable(
                            entries.into_iter().map(|ent| EntryTableRow {
                                entry: ent,
                                show_password: show,
                            }).collect::<Vec<_>>(),
                        );
                        println!("{}", Table::from(table));
                    }
                    Ok(())
                }
            }
        }
    }
}
