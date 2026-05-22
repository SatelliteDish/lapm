use lapm_core::{
    command::{
        entry::{
            DaemonEntry,
        },
    },
};
use clap::Subcommand;
use tabled::{
    Table,
    Tabled,
    settings::{
        Modify,
        Style,
        Width,
        object::Columns,
    },
};


pub struct EntryTableRow {
    pub entry: DaemonEntry,
    pub show_password: bool,
}

impl Tabled for EntryTableRow {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        let DaemonEntry {
            title,
            username,
            password,
            url,
            notes,
        } = &self.entry;

        vec![
            title.as_deref().unwrap_or_default().into(),
            username.into(),
            if self.show_password { password.into() } else { "********".into() },
            url.as_deref().unwrap_or_default().into(),
            notes.as_deref().unwrap_or_default().into(),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            "Title".into(),
            "Username".into(),
            "Password".into(),
            "URL".into(),
            "Notes".into(),
        ]
    }
}

pub struct EntryTable(pub Vec<EntryTableRow>);

impl From<EntryTable> for Table {
    fn from(value: EntryTable) -> Self {
        let mut table = Table::new(&value.0);
        table
            .with(Style::modern())
            .with(Modify::new(Columns::last()).with(Width::increase(13)));
        table
    }
}

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None)]
pub enum EntryCommand {
    Add{
        name: String,
        #[arg(short,long)]
        layer: String,
        #[arg(short,long)]
        url: Option<String>,
    },
    List{
        #[arg(short,long)]
        url: Option<String>,
        #[arg(long, default_value_t = false)]
        show: bool,
        #[arg(long, default_value_t = false)]
        copy: bool,
    },
}
