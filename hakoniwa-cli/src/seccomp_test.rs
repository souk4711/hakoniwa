use super::*;

fn contains_rule(rules: &[Rule], action: Action, sysname: &str) -> bool {
    rules
        .iter()
        .any(|r| r.action == action && r.sysname == sysname)
}

#[test]
fn test_load_audit() {
    let filter = load("audit").unwrap();
    let rules = filter.get_rules();
    assert!(rules.is_empty());
}

#[test]
fn test_load_seccomp() {
    let filter = load("podman").unwrap();
    let rules = filter.get_rules();

    assert!(contains_rule(&rules, Action::Allow, "accept"));
    assert!(contains_rule(&rules, Action::Allow, "brk"));
    assert!(contains_rule(&rules, Action::Allow, "read"));
    assert!(contains_rule(&rules, Action::Errno(1), "vm86"));
    assert!(contains_rule(&rules, Action::Errno(1), "vm86old"));

    #[cfg(target_arch = "x86_64")]
    {
        // includes#arches
        assert!(contains_rule(&rules, Action::Allow, "arch_prctl"));
        assert!(contains_rule(&rules, Action::Allow, "modify_ldt"));

        // includes#arches
        assert!(!contains_rule(&rules, Action::Allow, "s390_pci_mmio_read"));
        assert!(!contains_rule(&rules, Action::Allow, "s390_pci_mmio_write"));
    }

    // includes#caps
    assert!(contains_rule(&rules, Action::Allow, "sethostname"));
    assert!(contains_rule(&rules, Action::Allow, "clock_settime"));

    // excludes#caps
    assert!(!contains_rule(&rules, Action::Errno(1), "sethostname"));
    assert!(!contains_rule(&rules, Action::Errno(1), "clock_settime"));
}
