use std::env;
use std::path::{Path, PathBuf};

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            if let Ok(user) = env::var("USER") {
                return Path::new(&home).join(&user).join(rest);
            }
        }
    }
    PathBuf::from(path)
}
