use std::{fmt::Display, fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{gradient::Gradient, util};

const CONFIG_FILENAME: &str = ".psvconf.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortColors {
    FromGradient(Gradient),
    ColoredSubdisions(Vec<[f32; 3]>),
}

impl From<Gradient> for SortColors {
    fn from(value: Gradient) -> Self {
        SortColors::FromGradient(value)
    }
}

impl Display for SortColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::FromGradient(..) => "From Gradient",
            Self::ColoredSubdisions(..) => "Colored Subdivisions",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub scale_factor: f32,
    pub egui_opacity: u8,
    pub clear_color: [f32; 3],
    pub push_swap_path: Option<PathBuf>,
    pub sort_colors: SortColors,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scale_factor: 1.0,
            egui_opacity: 240,
            clear_color: [0.1, 0.2, 0.3],
            push_swap_path: None,
            sort_colors: SortColors::FromGradient(util::default_gradient()),
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
