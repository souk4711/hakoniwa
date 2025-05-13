use minijinja::Error;
use std::env;

pub(crate) fn env(name: String) -> Result<String, Error> {
    match env::var(name) {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}
