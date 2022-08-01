use nix::sched::CloneFlags;
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Namespaces {
    pub(crate) ns: Option<bool>,   // mount namespace
    pub(crate) uts: Option<bool>,  // uts namespace
    pub(crate) ipc: Option<bool>,  // ipc namespace
    pub(crate) pid: Option<bool>,  // pid namespace
    pub(crate) net: Option<bool>,  // network namespace
    pub(crate) user: Option<bool>, // user namespace
}

impl Namespaces {
    pub fn to_clone_flags(&self) -> CloneFlags {
        let mut flags = CloneFlags::empty();
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWNS, self.ns);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWUTS, self.uts);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWIPC, self.ipc);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWPID, self.pid);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWNET, self.net);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWUSER, self.user);
        flags
    }

    fn insert_clone_flag(flags: &mut CloneFlags, flag: CloneFlags, namespace: Option<bool>) {
        match namespace {
            Some(true) => flags.insert(flag),
            Some(_) => {}
            None => {}
        }
    }
}
