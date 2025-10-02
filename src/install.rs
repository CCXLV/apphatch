use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use uuid::Uuid;

use crate::desktop::Desktop;

fn extract_appimage(appimage_path: &str, extract_dir: &PathBuf) -> io::Result<()> {
    let new_path: PathBuf = extract_dir.join("App.AppImage");

    Command::new("mv")
        .arg(appimage_path)
        .arg(&new_path)
        .status()?;

    Command::new(&new_path).arg("--appimage-extract").status()?;

    println!("Extracted AppImage at {}", new_path.display());

    Ok(())
}

fn find_desktop_file(dir: &Path) -> Option<PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            let entry: fs::DirEntry = entry.ok()?;
            let path: PathBuf = entry.path();

            if path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext: &OsStr| ext == "desktop")
            {
                return Some(path);
            }
        }
    }
    None
}

fn find_executable(dir: &Path) -> io::Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if path.is_file() {
            let metadata: fs::Metadata = fs::metadata(&path)?;
            let permissions: fs::Permissions = metadata.permissions();
            if permissions.mode() & 0o111 != 0 {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}

fn find_icon(dir: &Path, file_name: &str) -> io::Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if path.is_file()
            && path
                .file_name()
                .map_or(false, |n: &OsStr| n == OsStr::new(file_name))
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn install_app(path: &String) -> Result<String, io::Error> {
    let short_id: String = Uuid::new_v4().simple().to_string()[..6].to_string();

    let temp_path: String = format!("/opt/{}", short_id);
    let temp_dir_path: PathBuf = PathBuf::from(temp_path);
    fs::create_dir(&temp_dir_path).expect("Failed to create temporary dir");

    extract_appimage(path, &temp_dir_path)?;

    let extracted_dir: PathBuf = temp_dir_path.join("squashfs-root");

    if let Some(path) = find_desktop_file(&extracted_dir) {
        let desktop: Result<Desktop, String> = Desktop::new(&path);
        let desktop: Desktop = match desktop {
            Ok(value) => value,
            Err(err) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        let app_dir_path = PathBuf::from(format!("/opt/{}", &desktop.name));
        let app_dir_path = match fs::exists(&app_dir_path) {
            Ok(_) => app_dir_path,
            Err(err) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        if let Err(err) = fs::rename(temp_dir_path, &app_dir_path) {
            return Err(err);
        }

        let exec_path: PathBuf = find_executable(&app_dir_path)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "exec not found"))?;

        let icon_name: &str = desktop
            .icon
            .as_deref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "icon name missing"))?;

        let icon_path: PathBuf = find_icon(&app_dir_path, icon_name)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "icon not found"))?;

        desktop.create_desktop(&exec_path, &icon_path)?;

        let _ = Command::new("update-desktop-database")
            .arg(
                std::env::var("HOME")
                    .map(|h: String| format!("{}/.local/share/applications", h))
                    .unwrap_or_else(|_| "/usr/share/applications".to_string()),
            )
            .output();

        Ok(format!("{} installed successfully at {}", &desktop.name, &app_dir_path.display()))
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .desktop file found",
        ));
    }
}
