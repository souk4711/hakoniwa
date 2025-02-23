#[cfg(test)]
mod container_test {
    use assertables::*;
    use std::env;
    use std::path::PathBuf;

    use hakoniwa::{Container, Namespace, Rlimit};

    fn current_dir() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    }

    fn customized_rootfs() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/rootfs"))
    }

    #[test]
    fn test_rootdir() {
        let dir = tempfile::tempdir().unwrap();
        let status = Container::new()
            .rootdir(&dir)
            .rootfs("/")
            .command("/bin/touch")
            .arg("newfile.txt")
            .status()
            .unwrap();
        assert!(status.success());
        assert!(dir.path().join("newfile.txt").exists());
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_rootdir_not_exists() {
        let mut container = Container::new();
        container.rootdir("/dir/not/exists");
    }

    #[test]
    fn test_rootfs() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/ls")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "bin\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "etc\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lib\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lib64\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "proc\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "sbin\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "usr\n");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("boot\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("dev\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("home\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("mnt\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("opt\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("root\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("run\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("sys\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("tmp\n"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("var\n"));
    }

    #[test]
    fn test_rootfs_dir_customized() {
        let output = Container::new()
            .rootfs(customized_rootfs())
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
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
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "lo: <LOOPBACK,UP,");

        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Network)
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(output.status.success());
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
        assert!(!output.status.success()); // Operation not permitted
        assert_contains!(String::from_utf8_lossy(&output.stderr), "hostname: ");

        let output = Container::new()
            .rootfs("/")
            .unshare(Namespace::Uts)
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_bindmount() {
        let output = Container::new()
            .rootfs("/")
            .bindmount(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,");

        let status = Container::new()
            .rootfs("/")
            .bindmount(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/touch")
            .args(["/myhome/Cargo.toml"])
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_bindmount_regular_file() {
        let output = Container::new()
            .rootfs("/")
            .bindmount(
                &current_dir().join("Cargo.toml").to_string_lossy(),
                "/myhome/Cargo.toml",
            )
            .command("/bin/stat")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "regular file");
    }

    #[test]
    fn test_bindmount_character_special_file() {
        let output = Container::new()
            .rootfs("/")
            .bindmount("/dev/null", "/mydev/null")
            .command("/bin/stat")
            .arg("/mydev/null")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stdout),
            "character special file"
        );
    }

    #[test]
    fn test_bindmount_container_path_overwrite() {
        let dir1 = customized_rootfs().join("bin");
        let dir2 = customized_rootfs().join("etc");
        let output = Container::new()
            .rootfs("/")
            .bindmount(&dir1.to_string_lossy(), "/mydir")
            .bindmount(&dir2.to_string_lossy(), "/mydir")
            .command("/bin/cat")
            .arg("/mydir/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Alpine Linux");
    }

    #[test]
    fn test_bindmount_container_path_nested() {
        let mut container = Container::new();
        let dir1 = customized_rootfs().join("bin");
        let dir2 = customized_rootfs().join("etc");
        let dir3 = customized_rootfs().join("lib");
        let dir4 = customized_rootfs().join("usr");
        container
            .rootfs("/")
            .bindmount(&dir1.to_string_lossy(), "/a1/b1/c1")
            .bindmount(&dir2.to_string_lossy(), "/a1")
            .bindmount(&dir3.to_string_lossy(), "/a1/b1/c2")
            .bindmount(&dir4.to_string_lossy(), "/a1/b1");

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/c1/busybox")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/c1/busybox\n"
        );

        let output = container
            .command("/bin/ls")
            .arg("/a1/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "/a1/os-release\n");

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/c2/ld-musl-x86_64.so.1")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/c2/ld-musl-x86_64.so.1\n"
        );

        let output = container
            .command("/bin/ls")
            .arg("/a1/b1/share/udhcpc/default.script")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/a1/b1/share/udhcpc/default.script\n"
        );
    }

    #[test]
    fn test_bindmount_ro() {
        let output = Container::new()
            .rootfs("/")
            .bindmount_ro(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " ro,");

        let output = Container::new()
            .rootfs("/")
            .bindmount_ro(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/touch")
            .args(["/myhome/Cargo.toml"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stderr),
            "Read-only file system"
        );
    }

    #[test]
    fn test_devfsmount() {
        let output = Container::new()
            .rootfs("/")
            .devfsmount("/mydev")
            .command("/bin/findmnt")
            .args(["-T", "/mydev"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "tmpfs");
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid");

        let output = Container::new()
            .rootfs("/")
            .devfsmount("/mydev")
            .command("/bin/ls")
            .arg("/mydev")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "null");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "zero");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "full");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "random");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "urandom");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "tty");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "stdin");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "stdout");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "stderr");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "shm");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "pts");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "ptmx");
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
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "tmpfs");
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid,nodev");

        let output = Container::new()
            .rootfs("/")
            .tmpfsmount("/mytmp")
            .command("/bin/touch")
            .arg("/mytmp/newfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = Container::new()
            .rootfs("/")
            .tmpfsmount("/mytmp")
            .uidmap(0)
            .command("/bin/touch")
            .arg("/mytmp/newfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_procfsmount() {
        let output = Container::new()
            .rootfs("/")
            .command("/bin/findmnt")
            .args(["-T", "/proc"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "proc");
        assert_contains!(
            String::from_utf8_lossy(&output.stdout),
            " rw,nosuid,nodev,noexec"
        );

        let output = Container::new()
            .rootfs("/")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/bin/cat");
    }

    #[test]
    fn test_procfsmount_local_procfs() {
        let output = Container::new()
            .rootfs("/")
            .bindmount("/proc", "/proc")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/sbin/init");
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
        assert!(output.status.success());
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
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n")
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
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "myhost\n");
    }

    #[test]
    fn test_setrlimit_fsize() {
        let output = Container::new()
            .rootfs("/")
            .devfsmount("/mydev")
            .tmpfsmount("/mytmp")
            .setrlimit(Rlimit::Fsize, 2, 2)
            .command("/bin/dd")
            .args([
                "if=/mydev/random",
                "of=/mytmp/output.txt",
                "count=1",
                "bs=4",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "File too large");
    }
}
