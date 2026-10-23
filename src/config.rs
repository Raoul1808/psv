use std::{fmt::Display, fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::gradient::Gradient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorProfile {
    pub name: String,
    pub clear_color: [f32; 4],
    pub colors: Gradient,
    #[serde(skip)]
    is_default: bool,
}

impl ColorProfile {
    pub fn new() -> Self {
        Self {
            name: "New Profile".into(),
            clear_color: [0.1, 0.2, 0.3, 0.0],
            colors: Gradient::from_slice(&[
                [0.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ]),
            is_default: false,
        }
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

impl Default for ColorProfile {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            clear_color: [0.1, 0.2, 0.3, 0.0],
            colors: Gradient::from_slice(&[
                [0.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 1.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ]),
            is_default: true,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VisualizerSideViewMode {
    #[default]
    None,
    Details,
    Visuals,
}

impl VisualizerSideViewMode {
    pub const ALL: [Self; 3] = [Self::None, Self::Details, Self::Visuals];
}

impl Display for VisualizerSideViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Details => write!(f, "Details"),
            Self::Visuals => write!(f, "Visuals"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppMode {
    #[default]
    Visualize,
    Benchmark,
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Visualize => write!(f, "Visualize"),
            AppMode::Benchmark => write!(f, "Benchmark"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub push_swap_path: Option<PathBuf>,
    pub selected_color_profile: usize,
    pub color_profiles: Vec<ColorProfile>,
    pub preferred_app_mode: AppMode,
    pub preferred_side_panel: VisualizerSideViewMode,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(CONFIG_PATH)?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
}

const CONFIG_PATH: &str = ".psv2conf.json";

impl Config {
    pub fn load_or_default() -> Self {
        let mut config = match load_config() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("config failed to load: {e}");
                log::warn!("using default config values as fallback");
                Config::default()
            }
        };
        config.color_profiles.insert(0, ColorProfile::default());
        config.selected_color_profile = config
            .selected_color_profile
            .clamp(0, config.color_profiles.iter().len() - 1);
        config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(CONFIG_PATH)?;
        let mut config = self.clone();
        config.color_profiles.retain(|c| !c.is_default);
        serde_json::to_writer_pretty(file, &config)?;
        log::info!("config saved!");
        Ok(())
    }

    pub fn save_with_log(&self) {
        if let Err(e) = self.save() {
            log::error!("error while saving config: {e}");
        }
    }

    pub fn try_get_current_profile(&self) -> Option<&ColorProfile> {
        self.color_profiles.get(self.selected_color_profile)
    }

    pub fn try_get_current_profile_mut(&mut self) -> Option<&mut ColorProfile> {
        self.color_profiles.get_mut(self.selected_color_profile)
    }

    pub fn get_current_profile(&self) -> &ColorProfile {
        self.try_get_current_profile().unwrap()
    }
}
