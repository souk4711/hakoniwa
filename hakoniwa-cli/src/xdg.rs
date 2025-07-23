mod user_dirs;

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

static USER_DIRS: LazyLock<Result<HashMap<String, String>>> = LazyLock::new(|| {
    let home = match env::home_dir() {
        Some(dir) => dir,
        None => return Err(anyhow!("home directory does not exist")),
    };

    let h = home.to_string_lossy().to_string();
    let mut m = HashMap::from([
        ("XDG_DESKTOP_DIR".to_string(), format!("{h}/Desktop")),
        ("XDG_DOWNLOAD_DIR".to_string(), format!("{h}/Downloads")),
        ("XDG_TEMPLATES_DIR".to_string(), format!("{h}/Templates")),
        ("XDG_PUBLICSHARE_DIR".to_string(), format!("{h}/Public")),
        ("XDG_DOCUMENTS_DIR".to_string(), format!("{h}/Documents")),
        ("XDG_MUSIC_DIR".to_string(), format!("{h}/Music")),
        ("XDG_PICTURES_DIR".to_string(), format!("{h}/Pictures")),
        ("XDG_VIDEOS_DIR".to_string(), format!("{h}/Videos")),
    ]);

    let path = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir).join("user-dirs.dirs"),
        None => home.join(".config/user-dirs.dirs"),
    };
    let entries = match user_dirs::UserDirsFile::new(&path).entries() {
        Ok(entries) => entries,
        Err(e) => return Err(e),
    };
    for entry in entries {
        let value = match entry.value.strip_prefix("$HOME/") {
            Some(v) => format!("{h}/{v}"),
            None => entry.value,
        };
        m.insert(entry.name, value);
    }
    Ok(m)
});

pub(crate) fn user_dir(name: &str) -> Result<String> {
    match USER_DIRS.as_ref() {
        Ok(dirs) => match dirs.get(&format!("XDG_{name}_DIR")) {
            Some(v) => Ok(v.to_string()),
            None => Ok("".to_string()),
        },
        Err(e) => Err(anyhow!(e)),
    }
}
