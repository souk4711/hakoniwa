#[cfg(test)]
mod container_test {
    use assertables::*;
    use regex::Regex;
    use std::env;
    use std::fs::{self, File};
    use std::path::PathBuf;

    use hakoniwa::{Container, Namespace, Pasta, Rlimit, Runctl};

    fn current_dir() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    }

    fn customized_rootfs_path() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/rootfs"
        ))
    }

    fn customized_scripts_path() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/scripts/"
        ))
    }

    fn userns_auto_uidmaps() -> Vec<(u32, u32, u32)> {
        let id = uzers::get_current_uid();
        let name = uzers::get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut idmaps = vec![(0, id, 1)];
        for line in fs::read_to_string("/etc/subuid").unwrap().lines() {
            let idmap = line.split(":").collect::<Vec<_>>();
            if idmap[0] == name {
                idmaps.push((1, idmap[1].parse().unwrap(), idmap[2].parse().unwrap()));
                break;
            }
        }

        assert_eq!(idmaps.len(), 2);
        idmaps
    }

    fn userns_auto_gidmaps() -> Vec<(u32, u32, u32)> {
        let id = uzers::get_current_gid();
        let name = uzers::get_current_username()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut idmaps = vec![(0, id, 1)];
        for line in fs::read_to_string("/etc/subgid").unwrap().lines() {
            let idmap = line.split(":").collect::<Vec<_>>();
            if idmap[0] == name {
                idmaps.push((1, idmap[1].parse().unwrap(), idmap[2].parse().unwrap()));
                break;
            }
        }

        assert_eq!(idmaps.len(), 2);
        idmaps
    }

    #[test]
    fn test_empty() {
        let output = Container::empty().command("/bin/ls").output().unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Cargo.toml\n");
    }

    #[test]
    fn test_unshare_net() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stdout),
            "1: lo: <LOOPBACK,UP,"
        );
        assert_contains!(String::from_utf8_lossy(&output.stdout), "2: ");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .unshare(Namespace::Network)
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "1: lo: <LOOPBACK>");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "2: ");
    }

    #[test]
    fn test_unshare_uts() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert!(!output.status.success()); // Operation not permitted
        assert_contains!(String::from_utf8_lossy(&output.stderr), "hostname: ");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .unshare(Namespace::Uts)
            .uidmap(0)
            .command("/bin/hostname")
            .arg("myhost")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_rootdir_customized() {
        let dir = tempfile::tempdir().unwrap();
        File::create(dir.path().join("myfile.txt")).unwrap();
        let output = Container::new()
            .rootdir(&dir)
            .rootfs("/")
            .unwrap()
            .command("/bin/ls")
            .arg("/myfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "/myfile.txt\n");
    }

    #[test]
    fn test_rootdir_not_exists() {
        let mut container = Container::new();
        let r = container
            .rootdir("/dir/not/exists")
            .command("/bin/ls")
            .output();
        assert_contains!(format!("{:?}", r.err()), "No such file or directory");
    }

    #[test]
    fn test_rootfs_local() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
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
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "boot\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "dev\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "home\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "mnt\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "opt\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "root\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "run\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "sys\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "tmp\n");
        assert_not_contains!(String::from_utf8_lossy(&output.stdout), "var\n");
    }

    #[test]
    fn test_rootfs_customized() {
        let output = Container::new()
            .rootfs(customized_rootfs_path())
            .unwrap()
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Alpine Linux");
    }

    #[test]
    fn test_rootfs_dir_not_exists() {
        let mut container = Container::new();
        let r = container.rootfs("/dir/not/exists");
        assert_contains!(format!("{:?}", r.err()), "No such file or directory");
    }

    #[test]
    fn test_rootfs_rdonly() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .command("/bin/touch")
            .arg("/myfile.txt")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stderr),
            "Read-only file system"
        );
    }

    #[test]
    fn test_bindmount_ro_dir() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " ro,nosuid");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
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
    fn test_bindmount_ro_regular_file() {
        let source = current_dir().join("Cargo.toml");
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&source.to_string_lossy(), "/myhome/Cargo.toml")
            .command("/bin/findmnt")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " ro,nosuid");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&source.to_string_lossy(), "/myhome/Cargo.toml")
            .command("/bin/touch")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stderr),
            "Read-only file system"
        );
    }

    #[test]
    fn test_bindmount_ro_container_path_overwrite() {
        let dir1 = customized_rootfs_path().join("bin");
        let dir2 = customized_rootfs_path().join("etc");
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&dir1.to_string_lossy(), "/mydir")
            .bindmount_ro(&dir2.to_string_lossy(), "/mydir")
            .command("/bin/cat")
            .arg("/mydir/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Alpine Linux");
    }

    #[test]
    fn test_bindmount_ro_container_path_nested() {
        let mut container = Container::new();
        let dir1 = customized_rootfs_path().join("bin");
        let dir2 = customized_rootfs_path().join("etc");
        let dir3 = customized_rootfs_path().join("lib");
        let dir4 = customized_rootfs_path().join("usr");
        container
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&dir1.to_string_lossy(), "/a1/b1/c1")
            .bindmount_ro(&dir2.to_string_lossy(), "/a1")
            .bindmount_ro(&dir3.to_string_lossy(), "/a1/b1/c2")
            .bindmount_ro(&dir4.to_string_lossy(), "/a1/b1");

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
    fn test_bindmount_ro_runc_error() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro("/bin", "dir/not/absolute")
            .command("/bin/true")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            output.status.reason,
            "mount target path must be absolute: dir/not/absolute"
        );
    }

    #[test]
    fn test_bindmount_rw_dir() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_rw(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/findmnt")
            .args(["-T", "/myhome"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_rw(&current_dir().to_string_lossy(), "/myhome")
            .command("/bin/touch")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_bindmount_rw_regular_file() {
        let source = current_dir().join("Cargo.toml");
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_rw(&source.to_string_lossy(), "/myhome/Cargo.toml")
            .command("/bin/findmnt")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_rw(&source.to_string_lossy(), "/myhome/Cargo.toml")
            .command("/bin/touch")
            .arg("/myhome/Cargo.toml")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_bindmount_rw_character_special_file() {
        let output = Container::new()
            .runctl(Runctl::MountFallback)
            .rootfs("/")
            .unwrap()
            .bindmount_rw("/dev/null", "/mydev/null")
            .command("/bin/findmnt")
            .arg("/mydev/null")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid");

        let output = Container::new()
            .runctl(Runctl::MountFallback)
            .rootfs("/")
            .unwrap()
            .bindmount_rw("/dev/null", "/mydev/null")
            .command("/bin/sh")
            .args(["-c", "echo 'myword' > /mydev/null"])
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_devfsmount_mount_options() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .devfsmount("/mydev")
            .command("/bin/findmnt")
            .args(["-T", "/mydev"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,");
    }

    #[test]
    fn test_devfsmount_default_devices() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
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
    fn test_devfsmount_writable() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .devfsmount("/mydev")
            .command("/bin/sh")
            .args(["-c", "echo 'myword' > /mydev/null"])
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_tmpfsmount_mount_options() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/mytmp")
            .command("/bin/findmnt")
            .args(["-T", "/mytmp"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "tmpfs");
        assert_contains!(String::from_utf8_lossy(&output.stdout), " rw,nosuid,nodev");
    }

    #[test]
    fn test_tmpfsmount_writable() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/mytmp")
            .command("/bin/touch")
            .arg("/mytmp/newfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/mytmp")
            .command("/bin/touch")
            .arg("/mytmp/newfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_procfsmount_mount_options() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
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
    }

    #[test]
    fn test_procfsmount_init_process() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/bin/cat");
    }

    #[test]
    fn test_procfsmount_local() {
        let output = Container::new()
            .runctl(Runctl::MountFallback)
            .rootfs("/")
            .unwrap()
            .bindmount_rw("/proc", "/proc")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/sbin/init");
    }

    #[test]
    fn test_file() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .file("/myfile.txt", "abc")
            .command("/bin/cat")
            .arg("/myfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "abc");
    }

    #[test]
    fn test_dir() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .dir("/tmp", 0o700)
            .command("/bin/stat")
            .args(["--printf", "%A", "/tmp"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "drwx------");
    }

    #[test]
    fn test_dir_chmod() {
        let source = current_dir().join("Cargo.toml");
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&source.to_string_lossy(), "/tmp/Cargo.toml")
            .command("/bin/stat")
            .args(["--printf", "%A", "/tmp"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "drwxr-xr-x");

        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro(&source.to_string_lossy(), "/tmp/Cargo.toml")
            .dir("/tmp", 0o700)
            .command("/bin/stat")
            .args(["--printf", "%A", "/tmp"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "drwx------");
    }

    #[test]
    fn test_symlink() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/tmp")
            .symlink("tmp", "/mytmp")
            .command("/bin/touch")
            .arg("/mytmp/newfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_uidmap() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .uidmap(0)
            .command("/bin/id")
            .arg("-u")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n");
    }

    #[test]
    fn test_gidmap() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .gidmap(0)
            .command("/bin/id")
            .arg("-g")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n");
    }

    #[test]
    fn test_uidmaps() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .uidmaps(&userns_auto_uidmaps())
            .command("/bin/cat")
            .arg("/proc/self/uid_map")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "         0 ");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "         1 ");
    }

    #[test]
    fn test_gidmaps() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .gidmaps(&userns_auto_gidmaps())
            .command("/bin/cat")
            .arg("/proc/self/gid_map")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "         0 ");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "         1 ");
    }

    #[test]
    fn test_gidmaps_setup_error() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .gidmaps(&[(0, 1000, 1), (1, 1, 10)])
            .command("/bin/cat")
            .arg("/proc/self/gid_map")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_is_match!(
            Regex::new(r"newgidmap: gid range .* not allowed").unwrap(),
            output.status.reason
        );
    }

    #[test]
    fn test_user_default_groups() {
        let mut container = Container::new();
        container.rootfs(customized_rootfs_path()).unwrap();
        container.uidmaps(&userns_auto_uidmaps());
        container.gidmaps(&userns_auto_gidmaps());

        let container = container.user("root", None, &[]);
        let output = container.command("/usr/bin/id").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "uid=0(root) gid=0(root) groups=0(root),1(bin),2(daemon),3(sys),4(adm),6(disk),10(wheel),11(floppy),20(dialout),26(tape),27(video)\n"
        );
    }

    #[test]
    fn test_user_specified_groups() {
        let mut container = Container::new();
        container.rootfs(customized_rootfs_path()).unwrap();
        container.uidmaps(&userns_auto_uidmaps());
        container.gidmaps(&userns_auto_gidmaps());

        let container = container.user("root", Some("root"), &[]);
        let output = container.command("/usr/bin/id").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "uid=0(root) gid=0(root)\n"
        );

        let container = container.user("root", Some("root"), &["wheel"]);
        let output = container.command("/usr/bin/id").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "uid=0(root) gid=0(root) groups=10(wheel)\n"
        );
    }

    #[test]
    fn test_hostname() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .unshare(Namespace::Uts)
            .hostname("myhost")
            .command("/bin/hostname")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "myhost\n");
    }

    #[test]
    fn test_network_pasta() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .unshare(Namespace::Network)
            .network(Pasta::default())
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stdout),
            "1: lo: <LOOPBACK,UP,"
        );
        assert_contains!(String::from_utf8_lossy(&output.stdout), "2: ");
    }

    #[test]
    fn test_network_pasta_runc_error() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_ro("/bin", "dir/not/absolute")
            .unshare(Namespace::Network)
            .network(Pasta::default())
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            output.status.reason,
            "mount target path must be absolute: dir/not/absolute"
        );
    }

    #[test]
    fn test_network_pasta_setup_error() {
        let mut network = Pasta::default();
        network.args(vec!["--myoption"]);
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .unshare(Namespace::Network)
            .network(network)
            .command("/bin/ip")
            .arg("link")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(output.status.reason, "pasta: unrecognized option");
    }

    #[test]
    fn test_setrlimit_fsize() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
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

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_fs_readable() {
        use hakoniwa::landlock::*;
        use std::str::FromStr;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/lib", FsAccess::from_str("r-x").unwrap());
        let output = Container::new()
            .rootfs(customized_rootfs_path())
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");

        ruleset.add_fs_rule("/etc", FsAccess::from_str("r--").unwrap());
        let output = Container::new()
            .rootfs(customized_rootfs_path())
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_fs_writable() {
        use hakoniwa::landlock::*;
        use std::str::FromStr;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/lib", FsAccess::from_str("r-x").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/tmp")
            .landlock_ruleset(ruleset.clone())
            .command("/bin/touch")
            .arg("/tmp/myfile.txt")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");

        ruleset.add_fs_rule("/tmp", FsAccess::from_str("-w-").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/tmp")
            .landlock_ruleset(ruleset.clone())
            .command("/bin/touch")
            .arg("/tmp/myfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_fs_executable() {
        use hakoniwa::landlock::*;
        use std::str::FromStr;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/lib", FsAccess::from_str("r--").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/echo")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");

        ruleset.add_fs_rule("/lib", FsAccess::from_str("r-x").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/echo")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_fs_rwx_dangerous() {
        use hakoniwa::landlock::*;
        use std::str::FromStr;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/lib", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/tmp", FsAccess::from_str("rw-").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/tmp")
            .landlock_ruleset(ruleset.clone())
            .command("/bin/sh")
            .args(["-c", "cp /bin/echo /tmp/echo && /tmp/echo"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");

        ruleset.add_fs_rule("/tmp", FsAccess::from_str("rwx").unwrap());
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .tmpfsmount("/tmp")
            .landlock_ruleset(ruleset.clone())
            .command("/bin/sh")
            .args(["-c", "cp /bin/echo /tmp/echo && /tmp/echo"])
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_fs_runc_error() {
        use hakoniwa::landlock::*;
        use std::str::FromStr;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/lib", FsAccess::from_str("r-x").unwrap());
        ruleset.add_fs_rule("/etc", FsAccess::from_str("r--").unwrap());
        ruleset.add_fs_rule("/nop", FsAccess::from_str("rwx").unwrap());
        let output = Container::new()
            .rootfs(customized_rootfs_path())
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/cat")
            .arg("/etc/os-release")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(
            String::from_utf8_lossy(&output.stderr),
            "landlock path must be exist: /nop"
        );
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_net_tcp_bind() {
        use hakoniwa::landlock::*;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::NET_TCP_BIND, CompatMode::Enforce);
        let output = Container::empty()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/python3")
            .arg(
                &customized_scripts_path()
                    .join("httpd-1s.py")
                    .to_string_lossy(),
            )
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Permission denied");

        ruleset.add_net_rule(8000, NetAccess::TCP_BIND);
        let output = Container::empty()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/python3")
            .arg(
                &customized_scripts_path()
                    .join("httpd-1s.py")
                    .to_string_lossy(),
            )
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_net_tcp_connect() {
        use hakoniwa::landlock::*;

        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::NET_TCP_CONNECT, CompatMode::Enforce);
        let output = Container::empty()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/aria2c")
            .args(["https://example.com", "--dry-run"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "Permission denied");

        ruleset.add_net_rule(443, NetAccess::TCP_CONNECT);
        let output = Container::empty()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/aria2c")
            .args(["https://example.com", "--dry-run"])
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "landlock")]
    #[test]
    fn test_landlock_empty() {
        use hakoniwa::landlock::*;

        let ruleset = Ruleset::default();
        let output = Container::new()
            .rootfs(customized_rootfs_path())
            .unwrap()
            .landlock_ruleset(ruleset.clone())
            .command("/bin/true")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "seccomp")]
    #[test]
    fn test_seccomp_errno() {
        use hakoniwa::seccomp::*;

        // let filter = Filter::new(Action::Errno(libc::EPERM));   // hangs when using Musl libc.
        let filter = Filter::new(Action::KillProcess);
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .seccomp_filter(filter)
            .command("/bin/echo")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(output.status.code, 128 + 31);
        assert_eq!(
            output.status.reason,
            "process(/bin/echo) received signal SIGSYS"
        );
        assert_eq!(output.status.exit_code, None);
    }

    #[cfg(feature = "seccomp")]
    #[test]
    fn test_seccomp_errno_allowlist() {
        use hakoniwa::seccomp::*;

        let mut filter = Filter::new(Action::Errno(libc::EPERM));
        filter.add_rule(Action::Allow, "access");
        filter.add_rule(Action::Allow, "arch_prctl");
        filter.add_rule(Action::Allow, "brk");
        filter.add_rule(Action::Allow, "close");
        filter.add_rule(Action::Allow, "execve");
        filter.add_rule(Action::Allow, "exit_group");
        filter.add_rule(Action::Allow, "fstat");
        filter.add_rule(Action::Allow, "getrandom");
        filter.add_rule(Action::Allow, "mmap");
        filter.add_rule(Action::Allow, "mprotect");
        filter.add_rule(Action::Allow, "munmap");
        filter.add_rule(Action::Allow, "newfstatat");
        filter.add_rule(Action::Allow, "openat");
        filter.add_rule(Action::Allow, "pread64");
        filter.add_rule(Action::Allow, "prlimit64");
        filter.add_rule(Action::Allow, "read");
        filter.add_rule(Action::Allow, "rseq");
        filter.add_rule(Action::Allow, "set_robust_list");
        filter.add_rule(Action::Allow, "set_tid_address");
        filter.add_rule(Action::Allow, "stat");
        filter.add_rule(Action::Allow, "write");
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .seccomp_filter(filter)
            .command("/bin/echo")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_runctl_rootfs_rw() {
        let output = Container::new()
            .runctl(Runctl::RootdirRW)
            .rootfs("/")
            .unwrap()
            .command("/bin/touch")
            .arg("/myfile.txt")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_runctl_mount_fallback() {
        let output = Container::new()
            .rootfs("/")
            .unwrap()
            .bindmount_rw("/proc", "/proc")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_contains!(output.status.reason, "EPERM");

        let output = Container::new()
            .runctl(Runctl::MountFallback)
            .rootfs("/")
            .unwrap()
            .bindmount_rw("/proc", "/proc")
            .command("/bin/cat")
            .arg("/proc/1/cmdline")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_runctl_allow_new_privs() {
        let output = Container::new()
            .runctl(Runctl::GetProcPidStatus)
            .rootfs("/")
            .unwrap()
            .command("/bin/true")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.status.proc_pid_status.unwrap().nonewprivs, 1);

        let output = Container::new()
            .runctl(Runctl::AllowNewPrivs)
            .runctl(Runctl::GetProcPidStatus)
            .rootfs("/")
            .unwrap()
            .command("/bin/true")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.status.proc_pid_status.unwrap().nonewprivs, 0);
    }
}
