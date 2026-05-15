use directories::ProjectDirs;
use tokio_util::sync::CancellationToken;
use arboard::Clipboard;
use std::{
    path::{Path, PathBuf},
    time::{Duration,Instant},
    sync::{Arc,Mutex},
};

mod config;
mod command;
mod entry;

mod layer;
use layer::{Layer,LayerState};

struct App {
    work_dir: PathBuf,
    config: config::AppConfig,
    clipboard: Clipboard,
}

impl App {
    pub fn add_layer(&mut self, name: String, password: String) -> Result<(), String> {
        let layer = Layer::create(name, &self.work_dir, password)?;
        self.config.layers.push(layer);
        config::write_config(&self.work_dir, &self.config)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_dirs = ProjectDirs::from("com", "lapm", "lapm")
        .ok_or("Could not open project directory".to_string())?;

    // Get Config
    let conf_dir: &Path = project_dirs.config_dir();
    let conf = config::get_config(conf_dir)?;


    let root_state = Arc::new(Mutex::new(
        App {
            config: conf,
            work_dir: conf_dir.to_path_buf(),
            clipboard: arboard::Clipboard::new()?,
        }
    ));

    // Setup daemon control
    let canc_tkn = CancellationToken::new();
    let clone_tkn = canc_tkn.clone();
    let state = root_state.clone();

    let local = tokio::task::LocalSet::new();

    local.spawn_local(command::handle_commands(root_state.clone()));

    local.run_until(async {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        // Daemon loop
        loop {
            tokio::select! {
                _ = clone_tkn.cancelled() => break,
                    _ = tokio::signal::ctrl_c() => break,
                    _ = interval.tick() => {
                    let mut state = state.lock()
                        .map_err(|e| e.to_string())?;
                    let now = Instant::now();
                    for layer in state.config.layers.iter_mut() {
                        if let LayerState::Open { last_used, timeout, .. } = layer.state {
                            if last_used + timeout < now {
                                layer.close();
                            }
                        }
                    }
                }
            }
        }
        Ok::<_, String>(())
    }).await?;

    println!("Cleanup!");
    Ok(())
}

