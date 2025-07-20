use crate::error::*;

pub(crate) fn to_string(option: Option<&str>) -> Result<String> {
    match option {
        Some(v) => Ok(v.to_string()),
        None => Err(EtcfsErrorKind::NotEnoughParts)?,
    }
}

pub(crate) fn to_u32(option: Option<&str>) -> Result<u32> {
    match option {
        Some(v) => Ok(v.parse().map_err(EtcfsErrorKind::StdNumParseIntError)?),
        None => Err(EtcfsErrorKind::NotEnoughParts)?,
    }
}
