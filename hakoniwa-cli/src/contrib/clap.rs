use clap::builder::styling::{AnsiColor, Styles};
use std::env;

pub(crate) fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

pub(crate) fn contains_flag(flag: &str) -> bool {
    for arg in std::env::args() {
        match arg.as_str() {
            "--" => return false,
            arg if arg == flag => return true,
            _ => {}
        }
    }
    false
}

pub(crate) fn parse_setenv<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    match s.find(['=', ':']) {
        Some(pos) => Ok((s[..pos].parse()?, s[pos + 1..].parse()?)),
        None => match env::var(s) {
            Ok(v) => Ok((s.parse()?, v.parse()?)),
            Err(_) => Ok((s.parse()?, "".parse()?)),
        },
    }
}

pub(crate) fn parse_key_val_colon_path<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    match s.find(':') {
        Some(pos) => Ok((s[..pos].parse()?, s[pos + 1..].parse()?)),
        None => Ok((s.parse()?, s.parse()?)),
    }
}
