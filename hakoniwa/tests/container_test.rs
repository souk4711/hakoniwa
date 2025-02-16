#[cfg(test)]
mod container_test {
    use assertables::*;
    use std::env;
    use std::path::PathBuf;

    use hakoniwa::{Container, MountOptions, Namespace, Rlimit};

    fn current_dir() -> String {
        env::current_dir().unwrap().to_str().unwrap().to_string()
    }

    fn customized_rootfs() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/rootfs"))
    }

    #[test]
    fn test_rootfs() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/ls")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "boot\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lost+found\n");
    }

    #[test]
    fn test_rootfs_dir_customized() {
        let output = Container::new()
            .rootfs(customized_rootfs())
            .command("/bin/ls")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "bin\ndev\netc\nlib\nsbin\nusr\nvar\n"
        );
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_rootfs_dir_not_exists() {
        let mut container = Container::new();
        container.rootfs("/dir/not/exists");
    }

    #[test]
    fn test_unshare_net() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/curl")
            .arg("https://example.com")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Example Domain");
        assert_contains!(String::from_utf8_lossy(&output.stderr), "");

        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Network)
            .command("/bin/curl")
            .arg("https://example.com")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), false);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "");
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Could not resolve");
    }

    #[test]
    fn test_unshare_uts() {
        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Uts)
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);

        let output = Container::new()
            .rootfs("/")
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), false);
        assert_contains!(String::from_utf8_lossy(&output.stderr), "hostname: ");
    }

    #[test]
    fn test_bindmount() {
        let output = Container::new()
            .rootfs("/")
            .bindmount(&current_dir(), "/myhome", MountOptions::empty())
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,");

        let status = Container::new()
            .rootfs(customized_rootfs())
            .bindmount(&current_dir(), "/myhome", MountOptions::empty())
            .command("/bin/touch")
            .args(["/myhome/Cargo.toml"])
            .status()
            .unwrap();
        assert_eq!(status.success(), true);
    }

    #[test]
    fn test_bindmount_ro() {
        let output = Container::new()
            .rootfs("/")
            .bindmount_ro(&current_dir(), "/myhome", MountOptions::empty())
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), " ro,");

        let output = Container::new()
            .rootfs(customized_rootfs())
            .bindmount_ro(&current_dir(), "/myhome", MountOptions::empty())
            .command("/bin/touch")
            .args(["/myhome/Cargo.toml"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), false);
        assert_contains!(
            String::from_utf8_lossy(&output.stderr),
            "Read-only file system"
        );
    }

    #[test]
    fn test_tmpfsmount() {
        let output = Container::new()
            .rootfs("/")
            .tmpfsmount("/mytmp")
            .command("/bin/findmnt")
            .args(&["-T", "/mytmp"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "tmpfs");
        assert_contains!(
            String::from_utf8_lossy(&output.stdout),
            " rw,nosuid,nodev,noexec"
        );
    }

    #[test]
    fn test_hostname() {
        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Uts)
            .hostname("myhost")
            .command("/bin/hostname")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "myhost\n");
    }

    #[test]
    fn test_uidmap() {
        let output = Container::new()
            .rootfs("/")
            .uidmap(0)
            .command("/bin/id")
            .arg("-u")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n")
    }

    #[test]
    fn test_gidmap() {
        let output = Container::new()
            .rootfs("/")
            .gidmap(0)
            .command("/bin/id")
            .arg("-g")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n")
    }

    #[test]
    fn test_setrlimit_fsize() {
        let status = Container::new()
            .rootfs("/")
            .setrlimit(Rlimit::Fsize, 2, 2)
            .command("/bin/dd")
            .args(&["if=/dev/random", "of=output.txt", "count=1", "bs=4"])
            .status()
            .unwrap();
        assert_eq!(status.success(), false);
        assert_eq!(status.code, 128 + 25);
        assert_eq!(status.reason, "waitpid(...) => Signaled(_, SIGXFSZ, _)");
        assert_eq!(status.exit_code, None);
    }
}
