use nix::sched::CloneFlags;

#[derive(Debug)]
pub(crate) struct Namespaces {
    pub(crate) ipc: Option<bool>,  // ipc namespace
    pub(crate) net: Option<bool>,  // network namespace
    pub(crate) ns: Option<bool>,   // mount namespace
    pub(crate) pid: Option<bool>,  // pid namespace
    pub(crate) user: Option<bool>, // user namespace
    pub(crate) uts: Option<bool>,  // uts namespace
}

impl Namespaces {
    pub(crate) fn to_clone_flags(&self) -> CloneFlags {
        let mut flags = CloneFlags::empty();
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWIPC, self.ipc);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWNET, self.net);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWNS, self.ns);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWPID, self.pid);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWUSER, self.user);
        Self::insert_clone_flag(&mut flags, CloneFlags::CLONE_NEWUTS, self.uts);
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

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            ipc: Some(true),  // create new ipc namespace
            net: Some(true),  // create new network namespace
            ns: Some(true),   // create new mount namespace
            pid: Some(true),  // create new pid namespace
            user: Some(true), // create new user namespace
            uts: Some(true),  // create new uts namespace
        }
    }
}
