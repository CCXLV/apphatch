use std::path::{Path, PathBuf};
use std::{fs, io};

use uuid::Uuid;

use crate::desktop::Desktop;
use crate::utils::{
    extract_appimage, find_desktop_file, find_executable, find_icon, flatten_squashfs_root,
    get_applications_dir,
};

fn backup_current_app(app_dir: &Path) -> Result<String, io::Error> {
    println!("Backing up the current app directory");
    let short_id: String = Uuid::new_v4().simple().to_string()[..6].to_string();
    let backup_dir_path: String = format!("/opt/{}", short_id);
    let backup_dir = PathBuf::from(format!("/opt/{}", short_id));

    fs::rename(&app_dir, &backup_dir)?;
    fs::create_dir_all(&app_dir)?;
    Ok(backup_dir_path)
}

fn revert(app_dir: &Path, backup_dir: &String) -> io::Result<()> {
    if app_dir.exists() {
        fs::remove_dir_all(app_dir)?;
    }
    fs::rename(&backup_dir, &app_dir)?;
    Ok(())
}

pub fn upgrade_app(name: &String, path: &String) -> Result<(), io::Error> {
    let parsed_app_name = &name.replace(" ", "-");
    println!("Starting upgrading app: {}", &parsed_app_name);

    let app_dir_path = format!("/opt/{}", &parsed_app_name);
    let app_dir = PathBuf::from(&app_dir_path);

    let applications_dir = get_applications_dir();
    let app_desktop_file_path = format!(
        "{}/{}.desktop",
        applications_dir.display(),
        &parsed_app_name
    );
    let app_desktop_file = PathBuf::from(&app_desktop_file_path);

    if !app_dir.exists() || !app_desktop_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "App at /opt/, or app's desktop ini file was not found",
        ));
    }

    let backup_dir_path = match backup_current_app(&app_dir) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };

    let appimage_path = PathBuf::from(path);
    let _ = match extract_appimage(&appimage_path, &app_dir) {
        Ok(_) => {}
        Err(err) => {
            let _ = revert(&app_dir, &backup_dir_path);
            return Err(err);
        }
    };

    let extracted_dir: PathBuf = app_dir.join("squashfs-root");

    let desktop_file: PathBuf = find_desktop_file(&extracted_dir)?.ok_or_else(|| {
        let _ = revert(&app_dir, &backup_dir_path);
        io::Error::new(io::ErrorKind::NotFound, "Desktop ini file not found")
    })?;

    let desktop: Result<Desktop, String> = Desktop::new(&desktop_file);
    let desktop: Desktop = match desktop {
        Ok(value) => value,
        Err(err) => {
            let _ = revert(&app_dir, &backup_dir_path);
            return Err(io::Error::new(io::ErrorKind::InvalidData, err));
        }
    };

    flatten_squashfs_root(&app_dir)?;

    let exec_path: PathBuf = find_executable(&app_dir)?.ok_or_else(|| {
        let _ = revert(&app_dir, &backup_dir_path);
        io::Error::new(io::ErrorKind::NotFound, "exec not found")
    })?;

    println!("Exec: {}", &exec_path.display());

    let icon_path: PathBuf = find_icon(&app_dir)?.ok_or_else(|| {
        let _ = revert(&app_dir, &backup_dir_path);
        io::Error::new(io::ErrorKind::NotFound, "icon not found")
    })?;

    println!("Icon: {}", &icon_path.display());

    println!("Updating current .desktop configuration");
    desktop.update_desktop(&desktop_file, &exec_path, &icon_path)?;

    println!("Successfully upgraded the app");

    Ok(())
}
