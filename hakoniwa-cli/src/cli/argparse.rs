use anyhow::Result;
use std::env;

pub(crate) fn contains_arg(arg: &str) -> bool {
    for a in env::args() {
        match a.as_str() {
            "--" => return false,
            a if a == arg => return true,
            _ => {}
        }
    }
    false
}

pub(crate) fn contains_arg_landlock() -> bool {
    for a in env::args() {
        match a.as_str() {
            "--" => return false,
            a if a.contains("--landlock") => return true,
            _ => {}
        }
    }
    false
}

pub(crate) fn contains_arg_raw() -> bool {
    match env::args().position(|arg| arg == "--") {
        Some(pos) => pos + 1 != env::args().len(),
        None => false,
    }
}

pub(crate) fn parse_rootdir(s: &str) -> Result<(String, String)> {
    match s.find(':') {
        Some(pos) => Ok((s[..pos].to_string(), s[pos + 1..].to_string())),
        None => Ok((s.to_string(), "".to_string())),
    }
}

pub(crate) fn parse_network(s: &str) -> Result<(String, String)> {
    match s.find(':') {
        Some(pos) => Ok((s[..pos].to_string(), s[pos + 1..].to_string())),
        None => Ok((s.to_string(), "".to_string())),
    }
}

pub(crate) fn parse_network_options(s: &str) -> Result<Vec<String>> {
    if s.is_empty() {
        Ok(vec![])
    } else {
        Ok(s.split(',').map(|s| s.to_string()).collect())
    }
}

pub(crate) fn parse_bindmount(s: &str) -> Result<(String, String)> {
    match s.find(':') {
        Some(pos) => Ok((s[..pos].to_string(), s[pos + 1..].to_string())),
        None => Ok((s.to_string(), s.to_string())),
    }
}

pub(crate) fn parse_symlink(s: &str) -> Result<(String, String)> {
    match s.find(':') {
        Some(pos) => Ok((s[..pos].to_string(), s[pos + 1..].to_string())),
        None => Ok((s.to_string(), s.to_string())),
    }
}

pub(crate) fn parse_setenv(s: &str) -> Result<(String, String)> {
    match s.find(['=']) {
        Some(pos) => Ok((s[..pos].to_string(), s[pos + 1..].to_string())),
        None => match env::var(s) {
            Ok(v) => Ok((s.to_string(), v.to_string())),
            Err(_) => Ok((s.to_string(), "".to_string())),
        },
    }
}

pub(crate) fn parse_landlock_net_ports(s: &str) -> Result<Vec<u16>> {
    if s.is_empty() {
        Ok(vec![])
    } else {
        Ok(s.split(',')
            .map(|e| e.to_string().parse::<u16>().unwrap_or(0))
            .filter(|e| *e != 0)
            .collect())
    }
}
