use directories::ProjectDirs;
use tokio_util::sync::CancellationToken;
use std::{
    path::{Path, PathBuf},
    time::{Duration,Instant},
    sync::{Arc,Mutex},
};

mod config;
mod command;
mod entry;

#[cfg(test)]
mod test_helpers;

mod layer;
use layer::{
    Layer,
    LayerState,
    OpenLayer,
};

struct App {
    work_dir: PathBuf,
    config: config::AppConfig,
}
type AppState = Arc<Mutex<App>>;

impl App {
    pub async fn add_layer(&mut self, name: String, password: String, timeout: Option<u64>) -> Result<(), String> {
        let layer = Layer::create(name, &self.work_dir, password, timeout).await
            .map_err(|e| e.to_string())?;
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
                        if let LayerState::Open(open) = &layer.state {
                            let OpenLayer { last_used, config, .. } = open;
                            if let Some(tout) = config.timeout {

                                if *last_used + tout < now {
                                    layer.close();
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok::<_, String>(())
    }).await?;
    Ok(())
}

