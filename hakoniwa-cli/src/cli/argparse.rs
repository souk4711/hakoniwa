use anyhow::Result;
use nix::unistd::{Gid, Uid};
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

pub(crate) fn parse_uidmap(s: &str) -> Result<(u32, u32, u32)> {
    let mut idmap: Vec<u32> = vec![];
    for e in s.split(':') {
        idmap.push(e.parse::<u32>()?)
    }
    match idmap.len() {
        0 => unreachable!("argparse::parse_uidmap"),
        1 => Ok((idmap[0], Uid::current().as_raw(), 1)),
        2 => Ok((idmap[0], idmap[1], 1)),
        _ => Ok((idmap[0], idmap[1], idmap[2])),
    }
}

pub(crate) fn parse_gidmap(s: &str) -> Result<(u32, u32, u32)> {
    let mut idmap: Vec<u32> = vec![];
    for e in s.split(':') {
        idmap.push(e.parse::<u32>()?)
    }
    match idmap.len() {
        0 => unreachable!("argparse::parse_gidmap"),
        1 => Ok((idmap[0], Gid::current().as_raw(), 1)),
        2 => Ok((idmap[0], idmap[1], 1)),
        _ => Ok((idmap[0], idmap[1], idmap[2])),
    }
}

pub(crate) fn parse_user(s: &str) -> Result<(String, Option<String>, Vec<String>)> {
    let parse_supplementary_groups = |s: &str| -> Vec<_> {
        match s.len() {
            0 => vec![],
            _ => s.split(',').map(|s| s.to_string()).collect(),
        }
    };
    let parts: Vec<_> = s.split(':').collect();
    match parts.len() {
        0 => unreachable!("argparse::parse_user"),
        1 => Ok((parts[0].to_string(), None, vec![])),
        2 => Ok((parts[0].to_string(), Some(parts[1].to_string()), vec![])),
        _ => Ok((
            parts[0].to_string(),
            Some(parts[1].to_string()),
            parse_supplementary_groups(parts[2]),
        )),
    }
}

pub(crate) fn parse_network(s: &str) -> Result<(String, Vec<String>)> {
    let parse_network_options = |s: &str| -> Vec<_> {
        match s.len() {
            0 => vec![],
            _ => s.split(',').map(|s| s.to_string()).collect(),
        }
    };
    let parts: Vec<_> = s.split(':').collect();
    match parts.len() {
        0 => unreachable!("argparse::parse_network"),
        1 => Ok((parts[0].to_string(), vec![])),
        _ => Ok((parts[0].to_string(), parse_network_options(parts[1]))),
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

pub(crate) fn parse_landlock_fs_paths(s: &str) -> Result<(u16, Vec<String>)> {
    let paths = s.split(&[',', ':']).map(|e| e.to_string()).collect();
    Ok((u16::MAX, paths))
}

pub(crate) fn parse_landlock_net_ports(s: &str) -> Result<(u16, Vec<u16>)> {
    let ports = s
        .split(',')
        .map(|e| e.to_string().parse::<u16>().unwrap_or(u16::MAX))
        .filter(|e| *e != u16::MAX)
        .collect();
    Ok((u16::MAX, ports))
}
