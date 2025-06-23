mod idmap;

use nix::unistd::Pid;

use crate::{error::*, Container};

pub(crate) use idmap::IdMap;

pub(crate) fn mainp_setup(_container: &Container, _child: Pid) -> Result<()> {
    Ok(())
}
