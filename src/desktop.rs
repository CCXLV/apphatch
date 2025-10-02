use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ini::ini;

fn get_name(map: &HashMap<String, HashMap<String, Option<String>>>) -> Option<&str> {
    let sec = map.get("desktop entry");
    let name = sec.and_then(|s| s.get("name")).and_then(|v| v.as_deref());

    name
}

fn non_empty(opt: Option<&str>) -> Option<String> {
    opt.and_then(|s: &str| {
        let t: &str = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}
pub struct Desktop {
    pub name: String,
    content: HashMap<String, HashMap<String, Option<String>>>,
}

impl Desktop {
    pub fn new(path: &PathBuf) -> Result<Self, String> {
        let content: HashMap<String, HashMap<String, Option<String>>> =
            ini!(path.to_string_lossy().as_ref());
        let name = get_name(&content);

        let name: Option<String> = non_empty(name);

        if name.is_none() {
            return Err("Required field missing: Name".to_string());
        }

        Ok(Self {
            name: name.unwrap(),
            content,
        })
    }

    pub fn create_desktop(
        &self,
        exec_path: impl AsRef<Path>,
        icon_path: impl AsRef<Path>,
    ) -> Result<(), io::Error> {
        // Determine the correct applications directory, even when run with sudo
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

        let destination_path: PathBuf =
            applications_dir.join(format!("{}.desktop", self.name.to_lowercase()));

        if destination_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Desktop ini file already exists",
            ));
        }

        // Build proper INI format under [Desktop Entry]
        let mut desktop_entry: HashMap<String, Option<String>> = self
            .content
            .get("desktop entry")
            .cloned()
            .unwrap_or_default();

        desktop_entry.insert(
            "exec".to_string(),
            Some(exec_path.as_ref().to_string_lossy().to_string()),
        );
        desktop_entry.insert(
            "icon".to_string(),
            Some(icon_path.as_ref().to_string_lossy().to_string()),
        );

        let mut lines: Vec<String> = Vec::new();
        lines.push("[Desktop Entry]".to_string());

        let mut keys: Vec<String> = desktop_entry.keys().cloned().collect();
        keys.sort();
        for key in keys {
            if let Some(Some(value)) = desktop_entry.get(&key).map(|v| v.as_ref()) {
                lines.push(format!("{}={}", canonicalize_key(&key), value));
            }
        }

        let desktop_content: String = lines.join("\n") + "\n";
        fs::write(destination_path, desktop_content)?;

        Ok(())
    }
}

fn canonicalize_key(key: &str) -> String {
    // Capitalize first letter and hyphen-separated segments
    let mut out: Vec<String> = Vec::new();
    for seg in key.split('-') {
        let mut chars = seg.chars();
        let cap = match chars.next() {
            None => String::new(),
            Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        };
        out.push(cap);
    }
    out.join("-")
}
