use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub fn uninstall_app(name: &str) -> Result<(), io::Error> {
    println!("Starting deleting the app {}", name);

    let app_dir_path: PathBuf = PathBuf::from(format!("/opt/{}", name.to_lowercase()));
    if !app_dir_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "App not found"));
    }

    let applications_dir_path = match (env::var("SUDO_USER"), env::var("USER")) {
        (Ok(sudo_user), _) => format!("/home/{}/.local/share/applications", sudo_user),
        (Err(_), Ok(user)) => format!("/home/{}/.local/share/applications", user),
        (Err(_), Err(_)) => "/usr/share/applications".to_string(),
    };

    let desktop_file_path = PathBuf::from(&applications_dir_path).join(format!("{}.desktop", name));
    println!("{}", &desktop_file_path.display());
    if !desktop_file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Desktop ini file not found",
        ));
    }

    println!("Removing desktop ini file...");
    fs::remove_file(&desktop_file_path)?;
    println!("Desktop ini file was removed");

    println!("Removing app dir...");
    fs::remove_dir_all(&app_dir_path)?;
    println!("App directory was removed");

    let _ = Command::new("update-desktop-database")
        .arg(&applications_dir_path)
        .output();

    println!("Desktop database was reloaded");

    Ok(())
}
