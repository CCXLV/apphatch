use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use uuid::Uuid;

use crate::desktop::Desktop;
use crate::utils::expand_tilde;

fn extract_appimage(appimage_path: &Path, extract_dir: &Path) -> io::Result<()> {
    let new_path: PathBuf = extract_dir.join("App.AppImage");

    fs::copy(appimage_path, &new_path)?;

    let mut perms = fs::metadata(&new_path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(&new_path, perms)?;

    let prev_dir: PathBuf = env::current_dir()?;
    env::set_current_dir(extract_dir)?;
    let status = Command::new(&new_path)
        .arg("--appimage-extract")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = env::set_current_dir(prev_dir);
    status?;

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
        if path.is_file() && path.extension().is_none() {
            let metadata: fs::Metadata = fs::metadata(&path)?;
            let permissions: fs::Permissions = metadata.permissions();
            if permissions.mode() & 0o111 != 0 {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}

fn find_icon(dir: &Path) -> io::Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext: &OsStr| ext == "png") {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn flatten_squashfs_root(app_dir: &Path) -> io::Result<()> {
    let src_dir: PathBuf = app_dir.join("squashfs-root");
    if !src_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(&src_dir)? {
        let entry: fs::DirEntry = entry?;
        let from_path: PathBuf = entry.path();
        let file_name = entry.file_name();
        let to_path: PathBuf = app_dir.join(file_name);

        if to_path.exists() {
            if to_path.is_dir() {
                fs::remove_dir_all(&to_path)?;
            } else {
                fs::remove_file(&to_path)?;
            }
        }

        fs::rename(&from_path, &to_path)?;
    }

    fs::remove_dir_all(&src_dir)?;
    Ok(())
}

fn cleanup(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn install_app(path: &String) -> Result<(), io::Error> {
    println!("Starting installing the app...");

    let short_id: String = Uuid::new_v4().simple().to_string()[..6].to_string();

    let temp_path: String = format!("/opt/{}", short_id);
    let temp_dir_path: PathBuf = PathBuf::from(temp_path);
    fs::create_dir(&temp_dir_path).expect("Failed to create temporary dir");
    println!("Created temporary dir: {}", &temp_dir_path.display());

    let expanded_input: PathBuf = expand_tilde(path);
    let canonical_input: PathBuf = expanded_input;

    extract_appimage(&canonical_input, &temp_dir_path)?;

    let extracted_dir: PathBuf = temp_dir_path.join("squashfs-root");

    if let Some(path) = find_desktop_file(&extracted_dir) {
        let desktop: Result<Desktop, String> = Desktop::new(&path);
        let desktop: Desktop = match desktop {
            Ok(value) => value,
            Err(err) => {
                let _ = cleanup(&temp_dir_path);
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        println!("App: {}", desktop.name);

        let app_dir_path: PathBuf = PathBuf::from(format!("/opt/{}", &desktop.name.to_lowercase()));
        let app_exists = fs::exists(&app_dir_path).expect("App already exists");
        if app_exists {
            let _ = cleanup(&temp_dir_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("App named {} already exists", &desktop.name.to_lowercase()).to_string(),
            ));
        }

        let _ = fs::rename(&temp_dir_path, &app_dir_path);

        flatten_squashfs_root(&app_dir_path)?;

        let exec_path: PathBuf = find_executable(&app_dir_path)?.ok_or_else(|| {
            let _ = cleanup(&temp_dir_path);
            let _ = cleanup(&app_dir_path);
            io::Error::new(io::ErrorKind::NotFound, "exec not found")
        })?;

        println!("Exec: {}", &exec_path.display());

        let icon_path: PathBuf = find_icon(&app_dir_path)?.ok_or_else(|| {
            let _ = cleanup(&temp_dir_path);
            let _ = cleanup(&app_dir_path);
            io::Error::new(io::ErrorKind::NotFound, "icon not found")
        })?;

        println!("Icon: {}", &icon_path.display());

        desktop.create_desktop(&exec_path, &icon_path)?;

        let _ = Command::new("update-desktop-database")
            .arg(
                std::env::var("HOME")
                    .map(|h: String| format!("{}/.local/share/applications", h))
                    .unwrap_or_else(|_| "/usr/share/applications".to_string()),
            )
            .output();

        println!(
            "{} installed successfully at {}",
            &desktop.name,
            &app_dir_path.display()
        );

        Ok(())
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .desktop file found",
        ));
    }
}
