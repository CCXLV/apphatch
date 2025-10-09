use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

pub fn extract_appimage(appimage_path: &Path, extract_dir: &Path) -> io::Result<()> {
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

pub fn find_desktop_file(dir: &Path) -> io::Result<Option<PathBuf>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry: fs::DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext: &OsStr| ext == "desktop")
            {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}

pub fn find_executable(dir: &Path) -> io::Result<Option<PathBuf>> {
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

pub fn find_icon(dir: &Path) -> io::Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if path.is_file()
            && path.extension().map_or(false, |ext: &OsStr| {
                ext == "png" || ext == "jpg" || ext == "svg"
            })
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn flatten_squashfs_root(app_dir: &Path) -> io::Result<()> {
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

        let appimage_path: PathBuf = app_dir.join("App.AppImage");
        if appimage_path.exists() {
            let _ = fs::remove_file(appimage_path);
        }

        fs::rename(&from_path, &to_path)?;
    }

    fs::remove_dir_all(&src_dir)?;
    Ok(())
}
