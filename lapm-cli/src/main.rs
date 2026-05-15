use lapm_core::{
    command::{
        entry::{
            DaemonEntryAddCommand, DaemonEntryListCommand,
        }, layer::{
            DaemonLayerAddCommand,
            DaemonLayerListCommand,
            DaemonLayerOpenCommand,
            LayerInfo,
        }
    },
    stream::IpcCommand as _,
};
use clap::{Parser,Subcommand};
use tabled::{Table, Tabled, settings::{Color, Modify, Style, Width, object::{Columns,Rows}}};

mod password;

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

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
enum LayerCommand {
    Add{ name: String },
    List,
    Open{ name: String },
}

struct LayerInfoTableRow(LayerInfo);
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

struct LayerInfoTable {
    pub layers: Vec<LayerInfoTableRow>,
}

impl From<LayerInfoTable> for Table {
    fn from(value: LayerInfoTable) -> Self {
        let mut table = Table::new(&value.layers);
        for (i, layer) in value.layers.iter().enumerate() {
            let color = if layer.0.open { Color::FG_GREEN } else { Color::FG_RED };
            table.with(Modify::new(Rows::one(i + 1)).with(color));
        }

        table
            .with(Style::modern())
            .with(Modify::new(Columns::last()).with(Width::increase(13)));
        table
    }
}

impl FromIterator<LayerInfoTableRow> for LayerInfoTable {
    fn from_iter<T: IntoIterator<Item = LayerInfoTableRow>>(iter: T) -> Self {
        Self { layers: iter.into_iter().collect() }
    }
}

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
enum EntryCommand {
    Add{
        name: String,
        #[arg(short,long)]
        layer: String
    },
    List,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        CliCommand::Layer{ layer } => {
            match layer {
                LayerCommand::Add{ name } => {
                    let password = password::get_and_confirm_password()?;

                    // Open stream AFTER password is received
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    DaemonLayerAddCommand{ name, password }
                        .send(&mut stream).await?;
                    Ok(())
                },
                LayerCommand::List => {
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    let res = DaemonLayerListCommand{}
                        .send(&mut stream).await?;
                    let table = res.layers.into_iter()
                        .map(|lyr| LayerInfoTableRow::from(lyr))
                        .collect::<LayerInfoTable>();
                    println!("{}", Table::from(table));

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
                }
            }
        },
        CliCommand::Entry { entry } => {
            match entry {
                EntryCommand::Add { name, layer } => {
                    let pwd = password::get_and_confirm_password()?;
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    DaemonEntryAddCommand{ name, password: pwd, layer }
                        .send(&mut stream).await?;
                    Ok(())
                },
                EntryCommand::List => {
                    let mut stream = lapm_core::stream::get_connection_stream().await?;
                    let entries = DaemonEntryListCommand{}
                        .send(&mut stream).await?;
                    println!("{entries:?}");
                    Ok(())
                }
            }
        }
    }
}
