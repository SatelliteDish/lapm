use clap::{Parser,Subcommand,Args};
use thiserror::Error;
mod password;
mod entry;
mod layer;

use entry::{
    EntryCommand,
    EntryError,
    handle_entry_command,
};
use layer::{
    LayerCommand,
    LayerError,
    handle_layer_command,
};



#[derive(Debug,Error)]
enum CliError {
    #[error("{0}")]
    LayerError(#[from]LayerError),
    #[error("{0}")]
    EntryError(#[from]EntryError),
}

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
async fn main() -> Result<(), CliError> {
    let args = Cli::parse();

    match args.command {
        CliCommand::Layer{ layer } => handle_layer_command(layer).await?,
        CliCommand::Entry { entry } => handle_entry_command(entry).await?,
    }

    Ok(())
}
