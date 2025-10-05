use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use uuid::Uuid;

use crate::desktop::Desktop;
use crate::utils::{
    expand_tilde, extract_appimage, find_desktop_file, find_executable, find_icon,
    flatten_squashfs_root, get_applications_dir,
};

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

    extract_appimage(&expanded_input, &temp_dir_path)?;

    let extracted_dir: PathBuf = temp_dir_path.join("squashfs-root");

    let desktop_file: PathBuf = find_desktop_file(&extracted_dir)?.ok_or_else(|| {
        let _ = cleanup(&temp_dir_path);
        io::Error::new(io::ErrorKind::NotFound, "Desktop ini file not found")
    })?;

    let desktop: Result<Desktop, String> = Desktop::new(&desktop_file);
    let desktop: Desktop = match desktop {
        Ok(value) => value,
        Err(err) => {
            let _ = cleanup(&temp_dir_path);
            return Err(io::Error::new(io::ErrorKind::InvalidData, err));
        }
    };

    println!("App: {}", desktop.name);

    let app_dir_name: &String = &desktop.name.to_lowercase().replace(" ", "-");
    let app_dir_path: PathBuf = PathBuf::from(format!("/opt/{}", app_dir_name));

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

    let applications_dir: PathBuf = get_applications_dir();
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir)
        .output();

    let _ = Command::new("chmod")
        .arg("-R")
        .arg("a+rwx")
        .arg(&app_dir_path)
        .output()
        .expect("failed to execute chmod");

    Ok(println!(
        "{} installed successfully at {}",
        &desktop.name,
        &app_dir_path.display()
    ))
}
