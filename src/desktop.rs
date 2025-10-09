use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ini::ini;
use uuid::Uuid;

use crate::utils::get_applications_dir;

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

fn backup_current_desktop(path: &Path) -> io::Result<PathBuf> {
    let parent: &Path = match path.parent() {
        Some(p) => p,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path has no parent",
            ));
        }
    };

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("desktop");
    let backup_name = format!("{}.bak-{}", stem, &Uuid::new_v4().simple().to_string()[..6]);
    let backup_path: PathBuf = parent.join(backup_name);

    fs::rename(path, &backup_path)?;
    Ok(backup_path)
}

fn revert_desktop(current_path: &Path, backup_path: &Path) -> Result<(), io::Error> {
    if !current_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Current .desktop configuration file doesnt exist",
        ));
    }

    fs::remove_file(&current_path)?;
    fs::rename(backup_path, current_path)?;

    Ok(())
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
        let applications_dir: PathBuf = get_applications_dir();

        let destination_path: PathBuf = applications_dir.join(format!(
            "{}.desktop",
            self.name.to_lowercase().replace(" ", "-")
        ));

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

    pub fn update_desktop(
        &self,
        current_path: impl AsRef<Path>,
        exec_path: impl AsRef<Path>,
        icon_path: impl AsRef<Path>,
    ) -> Result<(), io::Error> {
        let backup_path: PathBuf = backup_current_desktop(current_path.as_ref())?;
        println!("Backed up current .desktop configuration file");

        if let Err(err) = self.create_desktop(exec_path, icon_path) {
            let _ = revert_desktop(current_path.as_ref(), &backup_path);
            return Err(err);
        }

        println!(
            "Successfully updated current {} file",
            current_path.as_ref().display()
        );

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
