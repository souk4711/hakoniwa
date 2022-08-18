#[cfg(test)]
mod executor_test {
    use hakoniwa::{ExecutorResultStatus, Sandbox, SandboxPolicy};
    use nix::unistd::{Gid, Uid};

    fn sandbox() -> Sandbox {
        let mut sandbox = Sandbox::new();
        sandbox.with_policy(
            SandboxPolicy::from_str(
                r#"
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
]
    "#,
            )
            .unwrap(),
        );
        sandbox
    }

    #[test]
    fn test_namespace_ipc() {}

    #[test]
    fn test_namespace_net() {}

    #[test]
    fn test_namespace_ns() {}

    #[test]
    fn test_namespace_pid() {}

    #[test]
    fn test_namespace_user() {}

    #[test]
    fn test_namespace_uts() {}

    #[test]
    fn test_share_net_ns() {
        let mut executor = sandbox().command("ping", &["ping", "-c", "1", "127.0.0.1"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));

        let mut executor = sandbox().command("ping", &["ping", "-c", "1", "127.0.0.1"]);
        let result = executor.share_net_ns(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_uid() {
        let mut executor = sandbox().command("id", &["id", "-u"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            format!("{}\n", Uid::current().as_raw())
        );

        let mut executor = sandbox().command("id", &["id", "-u"]);
        let result = executor.uid(0).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), String::from("0\n"));
    }

    #[test]
    fn test_gid() {
        let mut executor = sandbox().command("id", &["id", "-g"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            format!("{}\n", Gid::current().as_raw())
        );

        let mut executor = sandbox().command("id", &["id", "-g"]);
        let result = executor.gid(0).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), String::from("0\n"));
    }

    #[test]
    fn test_hostname() {
        let mut executor = sandbox().command("hostname", &["hostname"]);
        let result = executor.hostname("test-hostname").run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            String::from("test-hostname\n")
        );
    }

    #[test]
    fn test_mount_new_tmpfs() {
        let mut executor = sandbox().command("ls", &["ls", "/tmp"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("No such file or directory"));

        let mut executor = sandbox().command("ls", &["ls", "/tmp"]);
        let result = executor.mount_new_tmpfs(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), String::from(""));
    }

    #[test]
    fn test_mount_new_devfs() {
        let mut executor = sandbox().command("ls", &["ls", "/dev"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("No such file or directory"));

        let mut executor = sandbox().command("ls", &["ls", "/dev"]);
        let result = executor.mount_new_devfs(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            String::from("null\nrandom\nurandom\nzero\n")
        );
    }

    #[test]
    fn test_setenv() {
        let mut executor = sandbox().command("env", &["env"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), String::from(""));

        let mut executor = sandbox().command("env", &["env"]);
        let result = executor.setenv("TEST-ENV", "12345678").run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            String::from("TEST-ENV=12345678\n")
        );
    }
}
