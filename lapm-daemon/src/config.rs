    fs, path::{Path, PathBuf}
};

use serde::{Serialize,Deserialize};
use crate::layer::{Layer,LayerState};

const CONFIG_FILE_NAME: &str = "config.bin";

#[derive(Serialize,Deserialize)]
struct AppConfigFileLayer {
    name: String,
    path: String,
}

impl From<AppConfigFileLayer> for Layer {
    fn from(value: AppConfigFileLayer) -> Self {
        Layer {
            name: value.name,
            path: PathBuf::from(value.path),
            state: LayerState::Closed,
        }
    }
}

impl From<Layer> for AppConfigFileLayer {
    fn from(value: Layer) -> Self {
        AppConfigFileLayer { name: value.name, path: value.path.to_str().unwrap().to_string() }
    }
}

#[derive(Serialize,Deserialize)]
struct AppConfigFile {
    layers: Vec<AppConfigFileLayer>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub layers: Vec<Layer>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            layers: vec![],
        }
    }
}

impl From<AppConfigFile> for AppConfig {
    fn from(value: AppConfigFile) -> Self {
        Self {
            layers: value.layers.
                into_iter()
                .map(|l| Layer::from(l))
                .collect(),
        }
    }
}


impl From<AppConfig> for AppConfigFile {
    fn from(value: AppConfig) -> Self {
        Self {
            layers: value.layers
                .into_iter()
                .map(|l| AppConfigFileLayer::from(l))
                .collect(),
        }
    }
}

pub fn get_config(path: &Path) -> Result<AppConfig,String> {
    let conf_path = path.join(CONFIG_FILE_NAME);
    let conf_exists = std::fs::exists(&conf_path).map_err(|e| format!("Failed to check if config exists: {e}"))?;

    if conf_exists {
        let conf_bytes = fs::read(&conf_path)
            .map_err(|e| format!("Failed to read conf bytes: {e}"))?;
        postcard::from_bytes::<AppConfigFile>(&conf_bytes)
            .map(|f| f.into())
            .map_err(|e| format!("Failed to parse bytes: {e}"))
    } else {
        let conf = AppConfig::default();
        fs::create_dir_all(path).map_err(|e| format!("Failed to create conf file: {e}"))?;
        write_config(path, &conf)?;
        Ok(conf)
    }
}

pub fn write_config(path: &Path, conf: &AppConfig) -> Result<(),String> {
    let conf_path = path.join(CONFIG_FILE_NAME);

    fs::write(conf_path, postcard::to_stdvec(&AppConfigFile::from(conf.clone())).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(())
}
