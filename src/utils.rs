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

pub fn get_applications_dir() -> PathBuf {
    let applications_dir: PathBuf = if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("applications")
    } else {
        let home: PathBuf = match (env::var("HOME"), env::var("SUDO_USER")) {
            (Ok(home), _) if !home.is_empty() && home != "/root" => PathBuf::from(home),
            (Ok(home), Ok(sudo_user)) if home == "/root" => {
                PathBuf::from(format!("/home/{}", sudo_user))
            }
            (Ok(home), _) => PathBuf::from(home),
            (Err(_), Ok(sudo_user)) => PathBuf::from(format!("/home/{}", sudo_user)),
            (Err(_), Err(_)) => PathBuf::from("/root"),
        };
        home.join(".local/share/applications")
    };
    applications_dir
}
