use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ini::ini;

fn get_fields(
    map: &HashMap<String, HashMap<String, Option<String>>>,
) -> (Option<&str>, Option<&str>) {
    let sec = map.get("desktop entry");
    let name = sec.and_then(|s| s.get("name")).and_then(|v| v.as_deref());
    let icon = sec.and_then(|s| s.get("icon")).and_then(|v| v.as_deref());

    (name, icon)
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

fn modify_fields(
    map: &HashMap<String, HashMap<String, Option<String>>>,
    field: &str,
    value: &str,
) -> HashMap<String, HashMap<String, Option<String>>> {
    let mut new_map: HashMap<String, HashMap<String, Option<String>>> = map.clone();
    if let Some(desktop_entry) = new_map.get_mut("desktop entry") {
        desktop_entry.insert(field.to_string(), Some(value.to_string()));
    }
    new_map
}

pub struct Desktop {
    pub name: String,
    pub icon: Option<String>,
    content: HashMap<String, HashMap<String, Option<String>>>,
}

impl Desktop {
    pub fn new(path: &PathBuf) -> Result<Self, String> {
        let content: HashMap<String, HashMap<String, Option<String>>> =
            ini!(path.to_string_lossy().as_ref());
        let (name, icon) = get_fields(&content);

        let name: Option<String> = non_empty(name);
        let icon: Option<String> = non_empty(icon);

        if name.is_none() {
            return Err("Required field missing: Name".to_string());
        }

        Ok(Self {
            name: name.unwrap(),
            icon,
            content,
        })
    }

    pub fn create_desktop(
        &self,
        exec_path: impl AsRef<Path>,
        icon_path: impl AsRef<Path>,
    ) -> Result<(), io::Error> {
        let mut destination_path = PathBuf::from(std::env::var("HOME").unwrap());
        destination_path.push(format!(".local/share/applications/{}.desktop", self.name));

        let destination_path = match fs::exists(&destination_path) {
            Ok(_) => destination_path,
            Err(err) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        let updated_content: HashMap<String, HashMap<String, Option<String>>> = modify_fields(
            &self.content,
            "exec",
            exec_path.as_ref().to_string_lossy().as_ref(),
        );
        let updated_content: HashMap<String, HashMap<String, Option<String>>> = modify_fields(
            &updated_content,
            "icon",
            icon_path.as_ref().to_string_lossy().as_ref(),
        );

        let desktop_content: String = format!("{:?}", updated_content);
        fs::write(destination_path, desktop_content)?;

        Ok(())
    }
}
