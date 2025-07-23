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

    let path = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => home.join(".config"),
    }
    .join("user-dirs.dirs");

    let entries = match user_dirs::UserDirsFile::new(&path).entries() {
        Ok(entries) => entries,
        Err(e) => return Err(e),
    };

    let mut m = HashMap::new();
    for entry in entries {
        let value = match entry.value.strip_prefix("$HOME/") {
            Some(value) => home.join(value).to_string_lossy().to_string(),
            None => entry.value,
        };
        m.insert(entry.name, value);
    }
    Ok(m)
});

pub(crate) fn user_dir(name: &str) -> Result<String> {
    let name = format!("XDG_{name}_DIR");
    match USER_DIRS.as_ref() {
        Ok(dirs) => Ok(dirs.get(&name).map_or("", |v| v).to_string()),
        Err(e) => Err(anyhow!(e)),
    }
}
