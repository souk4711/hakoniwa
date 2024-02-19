use std::{
    env,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub(crate) fn find_executable_path(prog: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let fullpath = dir.join(prog);
                match is_executable(&fullpath) {
                    true => Some(fullpath),
                    _ => None,
                }
            })
            .next()
    })
}

fn is_executable(path: &Path) -> bool {
    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    let permissions = metadata.permissions();
    metadata.is_file() && permissions.mode() & 0o111 != 0
}
