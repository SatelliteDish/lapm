use directories::ProjectDirs;
use tokio_util::sync::CancellationToken;
use std::{
    path::{Path, PathBuf},
    time::Duration,
    sync::{Arc,Mutex},
};

mod config;
mod command;
mod entry;

mod layer;
use layer::Layer;

struct App {
    work_dir: PathBuf,
    config: config::AppConfig,
}

impl App {
    pub fn add_layer(&mut self, name: String, password: String) -> Result<(), String> {
        let layer = Layer::create(name, &self.work_dir, password)?;
        self.config.layers.push(layer);
        config::write_config(&self.work_dir, &self.config)
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let project_dirs = ProjectDirs::from("com", "lapm", "lapm")
        .ok_or("Could not open project directory".to_string())?;

    // Get Config
    let conf_dir: &Path = project_dirs.config_dir();
    let conf = config::get_config(conf_dir)?;


    let root_state = Arc::new(Mutex::new(
        App {
            config: conf,
            work_dir: conf_dir.to_path_buf(),
        }
    ));

    command::handle_commands(root_state.clone()).await?;

    // Setup daemon control
    let canc_tkn = CancellationToken::new();
    let clone_tkn = canc_tkn.clone();

    // Daemon loop
    loop {
        tokio::select! {
        _ = clone_tkn.cancelled() => {
            break;
        }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // Tick
            }

        }
    }

    println!("Cleanup!");
    Ok(())
}

