use minijinja::Error;
use std::env;

pub(crate) fn env(value: String) -> Result<String, Error> {
    match env::var(value) {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}
