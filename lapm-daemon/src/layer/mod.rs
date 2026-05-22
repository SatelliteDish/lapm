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
use keepass::{Database, DatabaseKey, db::fields};
use thiserror::Error;
use derive_more::From;

use crate::entry::{
    Insert,
};

pub mod config;
use config::{
    LayerConfig,
    CONFIG_GROUP_NAME,
};


macro_rules! require_open {
    ($self:expr, |$open:ident| $body:expr) => {
        match &$self.state {
        LayerState::Open($open) => $body,
        LayerState::Closed => Err(LayerError::LayerClosed { name: $self.name.to_string() }),
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

        let config = LayerConfig {
            timeout: timeout.map(|tout| Duration::new(tout,0)),
            ..Default::default()
        };
        config.set_in_db(&mut db);

        let key = DatabaseKey::new().with_password(&password);
        let mut layer = Self {
            path: dir.join(format!("{name}.kdbx")).to_path_buf(),
            name,
            state: OpenLayer {
                last_used: Instant::now(),
                db,
                key,
                config,
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
            open.config = config;
            self.note_usage();
            self.save()?;
            Ok(())
        })
    }

    pub fn get_config(&self) -> Result<&LayerConfig, LayerError> {
        require_open!(self, |open| {
            Ok(&open.config)
        })
    }

    // Entry
    pub fn insert<'a>(&'a mut self, req: impl Insert) -> Result<(), LayerError> {
        require_open!(self, |mut open| {
            let OpenLayer { db, .. } = open;
            let mut root = db.root_mut();
            let _ = req.insert(&mut root);

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
