use interprocess::local_socket::{
    tokio::{
        Stream,
    },
};
use lapm_core::{
    IpcCommand as _,
    IpcError,
    command::entry::{
        DaemonEntryAddCommand,
        DaemonEntryCommand,
        DaemonEntryListCommand,
    },
};

use crate::{
    AppState,
    command::CommandError,
    entry::password::InsertPasswordEntry,
};


pub async fn execute_entry_command(app: AppState, command: DaemonEntryCommand, stream: &mut Stream) -> Result<(), CommandError> {
    match command {
        DaemonEntryCommand::Add(cmd)  => add_entry(app, stream, cmd).await,
        DaemonEntryCommand::List(cmd) => list_entries(app, stream, cmd).await,
    }
}

async fn add_entry(app: AppState, stream: &mut Stream, cmd: DaemonEntryAddCommand) -> Result<(), CommandError> {

    let DaemonEntryAddCommand{ entry, layer } = cmd;
    let mut state = app.lock()
        .map_err(|_| CommandError::MutexError)?;
    let found = state.config.layers.iter_mut()
        .find(|lyr| lyr.name.as_str() == layer.as_str());
    if let Some(layer) = found {
        DaemonEntryAddCommand::respond(
            layer.insert(InsertPasswordEntry::from(entry))
                .map_err(|e| {
                    eprint!("{e}");
                    IpcError::from(e)
                }),
            stream,
        ).await
            .map_err(|e| e.into())
    } else {
        DaemonEntryAddCommand::respond_err(
            IpcError::NotFound(format!("Could not find Layer \"{layer}\"")),
            stream,
        ).await
            .map_err(|e| e.into())
    }
}

async fn list_entries(app: AppState, stream: &mut Stream, cmd: DaemonEntryListCommand) -> Result<(), CommandError> {
    let mut state = app.lock()
        .map_err(|_| CommandError::MutexError)?;
    let DaemonEntryListCommand { url, copy } = cmd;
    let mut entries = state.config.layers
        .iter_mut().filter_map(|lyr| {
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
        .map_err(|e| e.into())
}
