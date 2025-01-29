use std::{fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::util;

const CONFIG_FILENAME: &str = ".psvconf.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub scale_factor: f32,
    pub egui_opacity: u8,
    pub push_swap_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scale_factor: 1.0,
            egui_opacity: 240,
            push_swap_path: None,
        }
    }
}

fn load_conf() -> Option<Config> {
    let file = File::open(CONFIG_FILENAME).ok()?;
    match serde_json::from_reader(file) {
        Ok(c) => Some(c),
        Err(e) => {
            if e.is_data() {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Config error")
                    .set_description("psv tried to load an invalid config file. If you manually edited the config file, please don't unless you know what you're doing. A new config file will be generated.")
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            }
            None
        }
    }
}

fn save_conf(config: &Config) -> anyhow::Result<()> {
    let file = File::create(CONFIG_FILENAME)?;
    serde_json::to_writer_pretty(file, config)?;
    Ok(())
}

impl Config {
    pub fn load() -> Config {
        let mut conf = load_conf().unwrap_or_default();
        if conf.push_swap_path.as_ref().is_some_and(|p| !p.exists()) {
            conf.push_swap_path = None;
        }
        if conf.push_swap_path.is_none() {
            conf.push_swap_path = util::detect_push_swap();
        }
        conf.egui_opacity = conf.egui_opacity.clamp(128, 255);
        conf
    }

    pub fn save(&self) {
        if let Err(e) = save_conf(self) {
            eprintln!("Failed to save config: {}", e);
        }
    }
}
