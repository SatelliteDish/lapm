use chrono::TimeDelta;
use lapm_core::{
    IpcError,
    command::{
        layer::LayerInfo,
        entry::DaemonEntry,
    },
};
use std::{fs::File, path::{Path,PathBuf}, time::Instant};
use keepass::{Database, DatabaseKey, db::fields};
use thiserror::Error;

use crate::entry::Entry;


#[derive(Debug,Clone,Error)]
pub enum LayerError<'a> {
    #[error("Layer \"{name}\" is closed. Please open it and try again.")]
    LayerClosed{ name: &'a str },
    #[error("Cannot access Database file at \"{path}\" to {operation}")]
    DbUnreachable{ path: &'a str, operation: &'a str },
}

impl<'a> From<LayerError<'a>> for IpcError {
    fn from(err: LayerError) -> Self {
        match &err {
            LayerError::LayerClosed {..} => IpcError::Unauthorized(err.to_string()),
            LayerError::DbUnreachable {..} => IpcError::OperationFailed(err.to_string()),
        }
    }
}

#[derive(Debug,Clone)]
pub struct Layer {
    pub name: String,
    pub path: PathBuf,
    pub state: LayerState,
}

impl Layer {
    pub fn create(name: String, dir: &Path, password: String) -> Result<Self,String> {
        let mut db = Database::new();
        db.meta.database_name = Some(name.clone());

        let key = DatabaseKey::new().with_password(&password);
        let mut layer = Self {
            path: dir.join(format!("{name}.kbdx")).to_path_buf(),
            name,
            state: LayerState::Open {
                public_usernames: false,
                timeout: TimeDelta::new(10, 10).unwrap(),
                last_used: Instant::now(),
                db,
                key,
            },
        };
        layer.save()
            .map_err(|e| e.to_string())?;

        Ok(layer)
    }

    pub fn is_open(&self) -> bool {
        match &self.state {
            LayerState::Open{..} => true,
            LayerState::Closed => false,
        }
    }

    pub fn open(&mut self, password: &str) -> Result<(),String> {
        if self.is_open() {
            return Ok(());
        }

        let mut file = File::open(&self.path)
            .map_err(|e| e.to_string())?;
        let key = DatabaseKey::new().with_password(password);
        let db = Database::open(&mut file, key.clone())
            .map_err(|e| e.to_string())?;

        self.state = LayerState::Open {
            public_usernames: false,
            timeout: TimeDelta::new(10,10).unwrap(),
            last_used: Instant::now(),
            db,
            key,
        };
        Ok(())
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), LayerError<'_>> {
        match &mut self.state {
            LayerState::Open { db, .. } => {
                let mut root = db.root_mut();
                let mut inserted = root.add_entry();

                inserted.set_unprotected(fields::USERNAME, &entry.username);
                inserted.set_unprotected(fields::PASSWORD, &entry.password);

                if let Some(title) = entry.title {
                    inserted.set_unprotected(fields::TITLE, &title);
                }
                if let Some(url) = entry.url {
                    inserted.set_unprotected(fields::URL, &url);
                }
                if let Some(notes) = entry.notes {
                    inserted.set_unprotected(fields::NOTES, &notes);
                }


                self.save()?;
                Ok(())
            },
            LayerState::Closed => Err(LayerError::LayerClosed { name: &self.name }),
        }
    }

    pub fn save(&mut self) -> Result<(), LayerError<'_>> {
        match &mut self.state {
            LayerState::Open { db, key, .. } => {
                db.save(
                    &mut File::open(&self.path)
                        .map_err(|_| LayerError::DbUnreachable {
                            path: &self.path.to_str().unwrap_or("Path contained invalid unicode"),
                            operation: "save",
                        })?,
                    key.clone(),
                ).map_err(|_| LayerError::DbUnreachable { // TODO: Add better error handling
                        path: &self.path.to_str().unwrap_or("Path contained invalid unicode"),
                        operation: "save"
                    })?;
                Ok(())
            },
            LayerState::Closed => Err(LayerError::LayerClosed { name: &self.name }),
        }
    }

    pub fn get_entries(&self) -> Result<Vec<DaemonEntry>, LayerError<'_>> {
        match &self.state {
            LayerState::Open { db, .. } => {
                let root = db.root();
                Ok(
                    root.entries()
                        .map(|ent| DaemonEntry {
                            title: ent.get(fields::TITLE).map(|ttl| ttl.to_string()),
                            username: ent.get(fields::USERNAME).unwrap().to_string(),
                            password: ent.get(fields::PASSWORD).unwrap().to_string(),
                            url: ent.get(fields::URL).map(|url| url.to_string()),
                            notes: ent.get(fields::NOTES).map(|nt| nt.to_string()),
                        }).collect::<Vec<_>>()

                )
            },
            LayerState::Closed => Err(LayerError::LayerClosed { name: &self.name }),
        }

    }
}

impl From<Layer> for LayerInfo {
    fn from(layer: Layer) -> Self {
        let is_open = layer.is_open();

        Self {
            name: layer.name,
            path: layer.path.to_str()
                .unwrap_or("Path contained invalid unicode")
                .to_string(),
            open: is_open,
        }
    }
}

#[derive(Debug,Clone)]
pub enum LayerState {
    Closed,
    Open{
        public_usernames: bool,
        timeout: TimeDelta,
        last_used: Instant,
        db: Database,
        key: DatabaseKey,
    },
}
