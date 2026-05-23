use lapm_core::{
    command::entry::{
            DaemonEntry, DaemonEntryAddCommand, DaemonEntryListCommand
        },
    stream::{IpcCommand as _, StreamError},
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
use thiserror::Error;

use crate::password::{self, PasswordError};


#[derive(Debug,Error)]
pub enum EntryError {
    #[error("{0}")]
    PasswordError(#[from]PasswordError),
    #[error("{0}")]
    StreamError(#[from]StreamError),
}
type EntryResult<T> = Result<T, EntryError>;


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

pub async fn handle_entry_command(command: EntryCommand) -> EntryResult<()> {
    match command {
        EntryCommand::Add { name, layer, url } => add_entry(name, layer, url).await,
        EntryCommand::List{ url, show, copy } => list_entries(url, show, copy).await
    }
}

async fn add_entry(name: String, layer: String, url: Option<String>) -> EntryResult<()> {
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
}

async fn list_entries(url: Option<String>, show_passwords: bool, copy: bool) -> EntryResult<()> {
            let mut stream = lapm_core::stream::get_connection_stream().await?;
            let entries = DaemonEntryListCommand{ url, copy }
                .send(&mut stream).await?;

            if !copy || show_passwords {
                let table = EntryTable(
                    entries.into_iter().map(|ent| EntryTableRow {
                        entry: ent,
                        show_password: show_passwords,
                    }).collect::<Vec<_>>(),
                );
                println!("{}", Table::from(table));
            }
            Ok(())
        }
