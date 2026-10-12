use std::{
    fs::{self, File},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub push_swap_path: Option<PathBuf>,
}

const CONFIG_PATH: &str = ".psv2conf.json";

impl Config {
    pub fn load_or_default() -> Result<Self, Box<dyn std::error::Error>> {
        if !fs::exists(CONFIG_PATH)? {
            return Ok(Self::default());
        }
        let file = File::open(CONFIG_PATH)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(CONFIG_PATH)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}
