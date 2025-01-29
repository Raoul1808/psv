use std::{fs, path::PathBuf};

pub fn detect_push_swap() -> Option<PathBuf> {
    if let Ok(path) = fs::canonicalize("push_swap") {
        Some(path)
    } else {
        None
    }
}
