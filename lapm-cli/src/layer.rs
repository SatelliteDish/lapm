use lapm_core::command::layer::LayerInfo;
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
use super::TimeoutArg;


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
