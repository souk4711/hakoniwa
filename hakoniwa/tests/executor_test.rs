#[cfg(test)]
mod executor_test {
    use hakoniwa::{ExecutorResultStatus, Sandbox, SandboxPolicy, Stdio};
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
    fn test_current_dir_default() {
        let mut executor = sandbox().command("pwd", &["pwd"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "/\n");
    }

    #[test]
    fn test_current_dir_custom() {
        let mut executor = sandbox().command("pwd", &["pwd"]);
        let result = executor.current_dir("/usr").unwrap().run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "/usr\n");
    }

    #[test]
    fn test_current_dir_error() {
        let mut executor = sandbox().command("pwd", &["pwd"]);
        let error = executor.current_dir("usr").unwrap_err();
        assert!(error.to_string().contains("should start with a /"));
    }

    #[test]
    #[ignore]
    fn test_namespace_ipc() {}

    #[test]
    #[ignore]
    fn test_namespace_net() {}

    #[test]
    #[ignore]
    fn test_namespace_ns() {}

    #[test]
    #[ignore]
    fn test_namespace_pid() {}

    #[test]
    #[ignore]
    fn test_namespace_user() {}

    #[test]
    #[ignore]
    fn test_namespace_uts() {}

    #[test]
    fn test_share_net_ns_false() {
        let mut executor = sandbox().command("ping", &["ping", "-c", "1", "127.0.0.1"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));
    }

    #[test]
    fn test_share_net_ns_true() {
        let mut executor = sandbox().command("ping", &["ping", "-c", "1", "127.0.0.1"]);
        let result = executor.share_net_ns(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_uid_default() {
        let mut executor = sandbox().command("id", &["id", "-u"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            format!("{}\n", Uid::current().as_raw())
        );
    }

    #[test]
    fn test_uid_custom() {
        let mut executor = sandbox().command("id", &["id", "-u"]);
        let result = executor.uid(0).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n");
    }

    #[test]
    fn test_gid_default() {
        let mut executor = sandbox().command("id", &["id", "-g"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            format!("{}\n", Gid::current().as_raw())
        );
    }

    #[test]
    fn test_gid_custom() {
        let mut executor = sandbox().command("id", &["id", "-g"]);
        let result = executor.gid(0).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n");
    }

    #[test]
    fn test_hostname_default() {
        let mut executor = sandbox().command("hostname", &["hostname"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "hakoniwa\n");
    }

    #[test]
    fn test_hostname_custom() {
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
    fn test_mount_new_tmpfs_false() {
        let mut executor = sandbox().command("ls", &["ls", "/tmp"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("No such file or directory"));
    }

    #[test]
    fn test_mount_new_tmpfs_true() {
        let mut executor = sandbox().command("ls", &["ls", "/tmp"]);
        let result = executor.mount_new_tmpfs(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
    }

    #[test]
    fn test_mount_new_devfs_false() {
        let mut executor = sandbox().command("ls", &["ls", "/dev"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("No such file or directory"));
    }

    #[test]
    fn test_mount_new_devfs_true() {
        let mut executor = sandbox().command("ls", &["ls", "/dev"]);
        let result = executor.mount_new_devfs(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            "null\nrandom\nurandom\nzero\n"
        );
    }

    #[test]
    fn test_setenv_default() {
        let mut executor = sandbox().command("env", &["env"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
    }

    #[test]
    fn test_setenv_custom() {
        let mut executor = sandbox().command("env", &["env"]);
        let result = executor.setenv("TEST-ENV", "12345678").run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            "TEST-ENV=12345678\n"
        );
    }

    #[test]
    #[ignore]
    fn test_limit_as() {}

    #[test]
    #[ignore]
    fn test_limit_core() {}

    #[test]
    #[ignore]
    fn test_limit_cpu() {}

    #[test]
    fn test_limit_fsize() {
        let prog = "dd";
        let argv = [prog, "if=/dev/random", "of=output.txt", "count=1", "bs=4"];
        let mut executor = sandbox().command(prog, &argv);
        let result = executor.mount_new_devfs(true).limit_fsize(Some(2)).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(1));
        assert!(String::from_utf8_lossy(&result.stderr).contains("File too large"));
    }

    #[test]
    fn test_limit_nofile() {
        let mut executor = sandbox().command("echo", &["echo"]);
        let result = executor.limit_nofile(Some(2)).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(127));
        assert!(String::from_utf8_lossy(&result.stderr).contains("cannot open shared object file"));
    }

    #[test]
    fn test_limit_walltime() {
        let mut executor = sandbox().command("sleep", &["sleep", "5"]);
        let result = executor.limit_walltime(Some(1)).run();
        assert_eq!(result.status, ExecutorResultStatus::TimeLimitExceeded);
        assert_eq!(result.exit_code, Some(128 + libc::SIGKILL));
    }

    #[test]
    #[ignore]
    fn test_seccomp_enable() {}

    #[test]
    #[ignore]
    fn test_seccomp_allow() {}

    #[test]
    fn test_stdout_initial() {
        let mut executor = sandbox().command("echo", &["echo", "Hako!"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "Hako!\n");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }

    #[test]
    fn test_stdout_inherit() {
        let mut executor = sandbox().command("echo", &["echo", "Hako!"]);
        let result = executor.stdout(Stdio::inherit_stdout()).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }

    #[test]
    fn test_stderr_initial() {
        let mut executor = sandbox().command("command404", &["command404"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
        assert_eq!(result.exit_code, None);
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        assert!(String::from_utf8_lossy(&result.stderr).contains("command not found"));
    }

    #[test]
    fn test_stderr_inherit() {
        let mut executor = sandbox().command("command404", &["command404"]);
        let result = executor.stderr(Stdio::inherit_stderr()).run();
        assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
        assert_eq!(result.exit_code, None);
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }
}
