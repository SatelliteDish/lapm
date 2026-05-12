use lapm_core::{IpcMessage as _, LapmCommand, LapmLayerCommand, LayerInfo, ListLayersResponse};
use clap::{Parser,Subcommand};
use tabled::{Tabled,Table,settings::{Style,object::{Columns,Rows},Modify,Width,Color}};

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
}

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
enum LayerCommand {
    Add{ name: String, password: String },
    List,
    Open{ name: String, password: String },
}

struct LayerInfoTable(LayerInfo);
impl From<LayerInfo> for LayerInfoTable {
    fn from(value: LayerInfo) -> Self {
        Self(value)
    }
}

impl Tabled for LayerInfoTable {
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

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Cli::parse();

    match args.command {
        CliCommand::Layer{ layer } => {
            match layer {
                LayerCommand::Add{ name, password } => {
                    let mut stream = lapm_core::get_connection_stream().await?;
                    let command = LapmCommand::Layer(
                        LapmLayerCommand::Add{ name, password },
                    );
                    command.send(&mut stream).await
                },
                LayerCommand::List => {
                    let mut stream = lapm_core::get_connection_stream().await?;
                    LapmCommand::Layer(LapmLayerCommand::List).send(&mut stream).await?;
                    let res = ListLayersResponse::receive(&mut stream).await?;
                    let rows = res.layers.into_iter()
                        .map(|lyr| LayerInfoTable::from(lyr));
                    let mut table = Table::new(rows.clone());
                    for (i, layer) in rows.enumerate() {
                        let color = if layer.0.open { Color::FG_GREEN } else { Color::FG_RED };
                        table.with(Modify::new(Rows::one(i + 1)).with(color));
                    }

                    table
                        .with(Style::modern())
                        .with(Modify::new(Columns::last()).with(Width::increase(13)));
                    println!("{table}");

                    Ok(())
                },
                LayerCommand::Open { name, password } => {
                    let mut stream = lapm_core::get_connection_stream().await?;
                    LapmCommand::Layer(
                        LapmLayerCommand::Open { name, password }
                    ).send(&mut stream).await
                }
            }
        },
    }

}
