use minijinja::Error;
use std::env;

pub(crate) fn is_env(value: String) -> bool {
    env::var(value).is_ok()
}

pub(crate) fn env(value: String) -> Result<String, Error> {
    match env::var(value) {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}
