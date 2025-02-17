#[cfg(test)]
mod container_test {
    use assertables::*;
    use std::env;
    use std::path::PathBuf;

    use hakoniwa::{Container, Namespace, Rlimit};

    fn current_dir() -> String {
        env::current_dir().unwrap().to_str().unwrap().to_string()
    }

    fn customized_rootfs() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/rootfs"))
    }

    #[test]
    fn test_root_dir() {
        let dir = tempfile::tempdir().unwrap();
        let status = Container::new()
            .root_dir(&dir)
            .rootfs("/")
            .command("/bin/touch")
            .arg("newfile.txt")
            .status()
            .unwrap();
        assert_eq!(status.success(), true);
        assert!(dir.path().join("newfile.txt").exists());
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_root_dir_not_exists() {
        let mut container = Container::new();
        container.root_dir("/dir/not/exists");
    }

    #[test]
    fn test_rootfs() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/ls")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "bin\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "etc\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lib\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "proc\n");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("dev\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("tmp\n"));
    }

    #[test]
    fn test_rootfs_dir_customized() {
        let output = Container::new()
            .rootfs(customized_rootfs())
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Alpine Linux");
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
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lo: <LOOPBACK,UP,");

        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Network)
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lo: <LOOPBACK>");
    }

    #[test]
    fn test_unshare_uts() {
        let output = Container::new()
            .rootfs("/")
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), false); // Operation not permitted
        assert_contains!(String::from_utf8_lossy(&output.stderr), "hostname: ");

        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Uts)
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
    }

    #[test]
    fn test_bindmount() {
        let output = Container::new()
            .rootfs("/")
            .bindmount(&current_dir(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,");

        let status = Container::new()
            .rootfs(customized_rootfs())
            .bindmount(&current_dir(), "/myhome")
            .command("/bin/touch")
            .args(["/myhome/Cargo.toml"])
            .status()
            .unwrap();
        assert_eq!(status.success(), true);
    }

    #[test]
    fn test_bindmount_container_path_same() {
        let dir1 = customized_rootfs().join("bin");
        let dir2 = customized_rootfs().join("etc");
        let output = Container::new()
            .rootfs("/")
            .bindmount(&dir1.into_os_string().into_string().unwrap(), "/mydir")
            .bindmount(&dir2.into_os_string().into_string().unwrap(), "/mydir")
            .command("/bin/cat")
            .arg("/mydir/os-release")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Alpine Linux");
    }

    #[test]
    fn test_mount_container_path_nested() {
        let mut container = Container::new();
        let dir1 = customized_rootfs().join("bin");
        let dir2 = customized_rootfs().join("etc");
        let dir3 = customized_rootfs().join("lib");
        let dir4 = customized_rootfs().join("usr");
        container
            .rootfs("/")
            .bindmount(&dir1.into_os_string().into_string().unwrap(), "/a1/b1/c1")
            .bindmount(&dir2.into_os_string().into_string().unwrap(), "/a1")
            .bindmount(&dir3.into_os_string().into_string().unwrap(), "/a1/b1/c2")
            .bindmount(&dir4.into_os_string().into_string().unwrap(), "/a1/b1");

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/c1/busybox")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/c1/busybox\n"
        );

        let output = container
            .command("/bin/ls")
            .arg("/a1/os-release")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "/a1/os-release\n");

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/c2/ld-musl-x86_64.so.1")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/c2/ld-musl-x86_64.so.1\n"
        );

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/share/udhcpc/default.script")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/share/udhcpc/default.script\n"
        );
    }

    #[test]
    fn test_bindmount_ro() {
        let output = Container::new()
            .rootfs("/")
            .bindmount_ro(&current_dir(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), " ro,");

        let output = Container::new()
            .rootfs(customized_rootfs())
            .bindmount_ro(&current_dir(), "/myhome")
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
            .args(["-T", "/mytmp"])
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
    fn test_procfsmount() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/bin/cat");
    }

    #[test]
    fn test_procfsmount_disable() {
        let output = Container::new()
            .rootfs("/")
            .bindmount("/proc", "/proc")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert_eq!(output.status.success(), true);
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/sbin/init");
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
        let output = Container::new()
            .rootfs("/")
            .bindmount("/dev", "/dev")
            .setrlimit(Rlimit::Fsize, 2, 2)
            .command("/bin/dd")
            .args(&["if=/dev/random", "of=output.txt", "count=1", "bs=4"])
            .output()
            .unwrap();
        assert_eq!(output.status.success(), false);
        assert_contains!(String::from_utf8_lossy(&output.stderr), "File too large");
    }
}
