use nix::{sched, Result};

use crate::namespaces::Namespaces;

pub fn init(namespaces: &Namespaces) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();
    sched::unshare(clone_flags)
}
