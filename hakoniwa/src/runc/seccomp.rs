use crate::runc::{error::*, nix};
use crate::Container;

pub(crate) fn load(_container: &Container) -> Result<()> {
    nix::set_no_new_privs()
}
