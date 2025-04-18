use std::{fs, path::PathBuf};

use crate::{config::ColorProfile, gradient::Gradient};

pub fn detect_push_swap() -> Option<PathBuf> {
    fs::canonicalize("push_swap").ok()
}

pub fn default_gradient() -> Gradient {
    let red = [1.0, 0.0, 0.0, 1.0];
    let yellow = [1.0, 1.0, 0.0, 1.0];
    let green = [0.0, 1.0, 0.0, 1.0];
    Gradient::from_slice(&[red, yellow, green])
}

pub fn default_profile() -> ColorProfile {
    ColorProfile {
        name: "Default".into(),
        colors: default_gradient().into(),
        clear_color: [0.1, 0.2, 0.3],
    }
}
