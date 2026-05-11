use chrono::TimeDelta;
use lapm_core::LayerInfo;
use std::{fs::File, path::{Path,PathBuf}, time::Instant};
use keepass::{Database, DatabaseKey};

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

        let layer = Self {
            path: dir.join(format!("{name}.kbdx")).to_path_buf(),
            name,
            state: LayerState::Open {
                public_usernames: false,
                timeout: TimeDelta::new(10, 10).unwrap(),
                last_used: Instant::now(),
                db,
            },
        };
        layer.save(&password)?;

        Ok(layer)
    }

    pub fn save(&self, password: &str) -> Result<(),String> {
        match &self.state {
            LayerState::Closed => Err("Cannot save closed vault".to_string()),
            LayerState::Open { public_usernames, timeout, last_used, db } => {
                println!("Saving");
                db.save(
                    &mut File::create(&self.path)
                        .map_err(|e| e.to_string())?,
                    DatabaseKey::new().with_password(password),
                ).map_err(|e| e.to_string())
            },
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
        let db = Database::open(&mut file, key)
            .map_err(|e| e.to_string())?;

        self.state = LayerState::Open {
            public_usernames: false,
            timeout: TimeDelta::new(10,10).unwrap(),
            last_used: Instant::now(),
            db,
        };
        Ok(())
    }
}

impl From<Layer> for LayerInfo {
    fn from(layer: Layer) -> Self {
        let is_open = layer.is_open();

        Self {
            name: layer.name,
            path: layer.path.to_str().unwrap().to_string(),
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
    },
}
