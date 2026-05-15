use lapm_core::{
    IpcError,
    command::{
        layer::LayerInfo,
        entry::DaemonEntry,
    },
};
use std::{
    fs::File,
    path::{Path,PathBuf},
    time::{Duration, Instant},
};
use keepass::{Database, DatabaseKey, db::{GroupMut, fields}};
use thiserror::Error;
use derive_more::From;

use crate::entry::Entry;

const CONFIG_GROUP_NAME: &str = "__config__";
const CONFIG_PUBLIC_USER_KEY: &str = "public_usernames";
const CONFIG_TIMEOUT_KEY: &str = "timeout";

macro_rules! require_open {
    ($self:expr, |$open:ident| $body:expr) => {
        match &$self.state {
        LayerState::Open($open) => $body,
        LayerState::Closed => Err(LayerError::LayerClosed { name: &$self.name }),
        }
    };
    ($self:expr, |mut $open:ident| $body:expr) => {
        match &mut $self.state {
        LayerState::Open($open) => $body,
        LayerState::Closed => Err(LayerError::LayerClosed { name: $self.name.to_string() }),
        }
    };
}

#[derive(Debug,Clone,Error)]
pub enum LayerError {
    #[error("Layer \"{name}\" is closed. Please open it and try again.")]
    LayerClosed{ name: String },
    #[error("Cannot access Database file at \"{path}\" to {operation}")]
    DbUnreachable{ path: String, operation: String },
}

impl From<LayerError> for IpcError {
    fn from(err: LayerError) -> Self {
        match &err {
            LayerError::LayerClosed {..} => IpcError::Unauthorized(err.to_string()),
            LayerError::DbUnreachable {..} => IpcError::OperationFailed(err.to_string()),
        }
    }
}

#[derive(Debug,Clone)]
pub struct LayerConfig {
    pub timeout: Option<Duration>,
    pub public_usernames: bool,
}

impl LayerConfig {
    fn get_by_key(conf_group: &mut GroupMut, key: &str) -> Option<String> {
        match &conf_group.entry_by_name_mut(key) {
            Some(ent) => {
                ent.get("value").map(|v| v.to_string())
            },
            None => None,
        }
    }

    fn set_by_key(conf_group: &mut GroupMut, key: &str, value: &str) {
        if let Some(mut ent) = conf_group.entry_by_name_mut(key) {
            ent.set_unprotected("value", value);
        } else {
            let mut ent = conf_group.add_entry();
            ent.set_unprotected(fields::TITLE, key);
            ent.set_unprotected("value", value);
        }
    }

    fn remove_by_key(conf_group: &mut GroupMut, key: &str) {
        if let Some(ent) = conf_group.entry_by_name_mut(key) {
            ent.remove();
        }
    }

    pub fn set_in_db(self, db: &mut Database) {
        let mut root = db.root_mut();
        let mut config_group =  match root.group_by_name_mut(CONFIG_GROUP_NAME) {
            Some(gp) => gp,
            None => {
                let mut group = root.add_group();
                group.name = CONFIG_GROUP_NAME.to_string();
                group
            },
        };

        Self::set_by_key(
            &mut config_group,
            CONFIG_PUBLIC_USER_KEY,
            &self.public_usernames.to_string(),
        );

        match self.timeout {
            Some(timeout) => Self::set_by_key(
                &mut config_group,
                CONFIG_TIMEOUT_KEY,
                &timeout.as_secs().to_string(),
            ),
            None => Self::remove_by_key(&mut config_group, CONFIG_TIMEOUT_KEY),
        }
    }

    pub fn from_db(db: &mut Database) -> Option<Self> {
        let mut root = db.root_mut();
        let mut config_group = root.group_by_name_mut(CONFIG_GROUP_NAME)?;

        Some(Self {
            public_usernames: Self::get_by_key(&mut config_group, CONFIG_PUBLIC_USER_KEY)
                .map(|str| if str == "true" { true } else { false })
                .unwrap_or(false),
            timeout: Self::get_by_key(&mut config_group, CONFIG_TIMEOUT_KEY)
                .map(|st| Duration::new(st.parse::<u64>().unwrap_or(60),0)),
        })
    }
}

#[derive(Debug,Clone)]
pub struct OpenLayer {
    pub last_used: Instant,
    pub db: Database,
    pub key: DatabaseKey,
    pub config: LayerConfig,
}

#[derive(Debug,Clone,From)]
pub enum LayerState {
    Closed,
    Open(#[from]OpenLayer),
}


#[derive(Debug,Clone)]
pub struct Layer {
    pub name: String,
    pub path: PathBuf,
    pub state: LayerState,
}

impl Layer {
    // Constructors
    pub async fn create(name: String, dir: &Path, password: String, timeout: Option<u64>) -> Result<Self,String> {
        let mut db = Database::new();
        db.meta.database_name = Some(name.clone());
        let mut db_root = db.root_mut();
        let mut config_group = db_root.add_group();
        config_group.name = CONFIG_GROUP_NAME.to_string();
        if let Some(tout) = timeout {
            let mut timeout_entry = config_group.add_entry();
            timeout_entry.set_unprotected(fields::TITLE, "timeout");
            timeout_entry.set_unprotected("value", tout.to_string());
        }

        let key = DatabaseKey::new().with_password(&password);
        let mut layer = Self {
            path: dir.join(format!("{name}.kdbx")).to_path_buf(),
            name,
            state: OpenLayer {
                last_used: Instant::now(),
                db,
                key,
                config: LayerConfig {
                    timeout: timeout.map(|tout| Duration::new(tout, 0)),
                    public_usernames: false,
                }
            }.into(),
        };
        layer.save()
            .map_err(|e| e.to_string())?;

        Ok(layer)
    }



    // Utility
    fn note_usage(&mut self) {
        if let LayerState::Open(open) = &mut self.state {
            open.last_used = Instant::now();
        }
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
        let db_root = db.root();
        let config_group = db_root.group_by_name(CONFIG_GROUP_NAME)
            .ok_or("Vault has no config")?;
        let timeout_entry = config_group.entry_by_name("timeout");
        let timeout = if let Some(tent) = timeout_entry {
            let value = tent.get("value");
            if let Some(tout) = value {
                Some(tout.parse::<u64>().map_err(|e| e.to_string())?)
            } else {
                None
            }
        } else { None };

        self.state = OpenLayer {
            last_used: Instant::now(),
            db,
            key,
            config: LayerConfig {
                timeout: timeout.map(|tout| Duration::new(tout, 0)),
                public_usernames: false,
            }
        }.into();
        Ok(())
    }

    pub fn close(&mut self) {
        self.state = LayerState::Closed;
    }

    pub fn save(&mut self) -> Result<(), LayerError> {
        require_open!(self, |mut open| {
            let OpenLayer { db, key, .. } = open;
            let mut fd = File::create(&self.path)
                    .map_err(|e| {
                        eprint!("{e}");
                        LayerError::DbUnreachable {
                            path: self.path
                                .to_str().unwrap_or("Path contained invalid unicode")
                                .to_string(),
                            operation: "save".to_string(),
                        }})?;
            db.save(
                &mut fd,
                key.clone(),
            ).map_err(|e| {
                    eprint!("{e}");
                    LayerError::DbUnreachable { // TODO: Add better error handling
                        path: self.path
                            .to_str().unwrap_or("Path contained invalid unicode")
                            .to_string(),
                        operation: "save".to_string(),
                    }})?;
            self.note_usage();
            Ok(())
        })
    }

    // Config
    pub fn set_config(&mut self, config: LayerConfig) -> Result<(), LayerError> {
        require_open!(self, |mut open| {
            config.set_in_db(&mut open.db);
            self.note_usage();
            Ok(())
        })
    }

    // Entry
    pub fn add_entry(&mut self, entry: Entry) -> Result<(), LayerError> {
        require_open!(self, |mut open| {
            let OpenLayer { db, .. } = open;
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


            self.note_usage();
            self.save()?;
            Ok(())
        })
    }

    pub fn get_entries(&mut self) -> Result<Vec<DaemonEntry>, LayerError> {
        require_open!(self, |mut open| {
            let OpenLayer { db, .. } = open;
            let root = db.root();
            let entries = root.entries()
                    .map(|ent| DaemonEntry {
                        title: ent.get(fields::TITLE).map(|ttl| ttl.to_string()),
                        username: ent.get(fields::USERNAME).unwrap().to_string(),
                        password: ent.get(fields::PASSWORD).unwrap().to_string(),
                        url: ent.get(fields::URL).map(|url| url.to_string()),
                        notes: ent.get(fields::NOTES).map(|nt| nt.to_string()),
                    }).collect::<Vec<_>>();
            self.note_usage();

            Ok(entries)
        })
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
