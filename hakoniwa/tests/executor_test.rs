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
    fn test_namespace_share_net_ns() {
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
    fn test_namespace_uid() {
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
    fn test_namespace_gid() {
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
}
