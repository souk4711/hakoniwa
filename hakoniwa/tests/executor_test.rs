#[cfg(test)]
mod executor_test {
    use hakoniwa::{ExecutorResultStatus, Sandbox, SandboxPolicy, SeccompAction, Stdio};
    use nix::unistd::{self, Gid, Uid};
    use std::env;

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
    fn test_container_root_dir_default() {
        let mut executor = sandbox().command("/bin/true", &["true"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_container_root_dir_custom() {
        let mut executor = sandbox().command("/bin/true", &["true"]);
        let dir = env::current_dir().unwrap().join("test-container-root-dir");
        let result = executor.container_root_dir(dir.clone()).unwrap().run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_container_root_dir_custom_relative_path() {
        let mut executor = sandbox().command("/bin/true", &["true"]);
        let dir = "test-container-root-dir-relative-path";
        let result = executor.container_root_dir(dir).unwrap().run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
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
    fn test_share_uts_ns_false() {
        let mut executor = sandbox().command("hostname", &["hostname"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "hakoniwa\n");
    }

    #[test]
    fn test_share_uts_ns_true() {
        let mut executor = sandbox().command("hostname", &["hostname"]);
        let result = executor.share_uts_ns(true).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim_end(),
            unistd::gethostname().unwrap().to_str().unwrap()
        );
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
    fn test_mount_tmpfs() {
        let mut executor = sandbox().command("findmnt", &["findmnt", "-T", "/mytmp"]);
        let result = executor.mount_tmpfs("/mytmp").unwrap().run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert!(String::from_utf8_lossy(&result.stdout).contains("tmpfs"));
    }

    #[test]
    fn test_ro_bind() {
        let mut executor = sandbox().command("touch", &["touch", "Cargo.toml"]);
        let result = executor
            .ro_bind(".", "/hako")
            .unwrap()
            .current_dir("/hako")
            .unwrap()
            .run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(1));
        assert!(String::from_utf8_lossy(&result.stderr).contains("Read-only file system"));
    }

    #[test]
    fn test_rw_bind() {
        let mut executor = sandbox().command("touch", &["touch", "Cargo.toml"]);
        let result = executor
            .rw_bind(".", "/hako")
            .unwrap()
            .current_dir("/hako")
            .unwrap()
            .run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_new_file() {
        let mut executor = sandbox().command("cat", &["cat", "/tmp/a/b/c.txt"]);
        let result = executor.new_file("/tmp/a/b/c.txt", "Hako!").unwrap().run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "Hako!");
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
    fn test_limit_cpu() {
        let mut executor = sandbox().command("bc", &["bc", "-l"]);
        let result = executor
            .limit_cpu(Some(1))
            .stdin(Stdio::from("scale=5000;a(1)*4\n"))
            .run();
        assert_eq!(result.status, ExecutorResultStatus::TimeLimitExceeded);
        assert_eq!(result.exit_code, Some(128 + libc::SIGKILL));
    }

    #[test]
    fn test_limit_fsize() {
        let prog = "dd";
        let argv = [prog, "if=/dev/random", "of=output.txt", "count=1", "bs=4"];
        let mut executor = sandbox().command(prog, &argv);
        let result = executor
            .rw_bind("/dev/random", "/dev/random")
            .unwrap()
            .limit_fsize(Some(2))
            .run();
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
    fn test_seccomp_dismatch_action_kill() {
        let mut executor = sandbox().command("echo", &["echo"]);
        let result = executor.seccomp_enable().run();
        assert_eq!(result.status, ExecutorResultStatus::RestrictedFunction);
        assert_eq!(result.exit_code, Some(128 + libc::SIGSYS));
    }

    #[test]
    fn test_seccomp_dismatch_action_allow_ok() {
        let mut executor = sandbox().command("echo", &["echo"]);
        let result = executor
            .seccomp_enable()
            .seccomp_dismatch_action(SeccompAction::Allow)
            .run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_seccomp_dismatch_action_allow_rfe() {
        let mut executor = sandbox().command("echo", &["echo"]);
        let result = executor
            .seccomp_enable()
            .seccomp_dismatch_action(SeccompAction::Allow)
            .seccomp_syscall_add("write")
            .unwrap()
            .run();
        assert_eq!(result.status, ExecutorResultStatus::RestrictedFunction);
        assert_eq!(result.exit_code, Some(128 + libc::SIGSYS));
    }

    #[test]
    fn test_seccomp_dismatch_action_log() {
        let mut executor = sandbox().command("echo", &["echo"]);
        let result = executor
            .seccomp_enable()
            .seccomp_dismatch_action(SeccompAction::Log)
            .run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
    }

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
        let result = executor.stdout(Stdio::inherit()).run();
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
        let result = executor.stderr(Stdio::inherit()).run();
        assert_eq!(result.status, ExecutorResultStatus::SandboxSetupError);
        assert_eq!(result.exit_code, None);
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }

    #[test]
    fn test_stdin_initial() {
        let mut executor = sandbox().command("cat", &["cat"]);
        let result = executor.run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }

    #[test]
    fn test_stdin_from_string() {
        let mut executor = sandbox().command("cat", &["cat"]);
        let result = executor.stdin(Stdio::from("Hako!")).run();
        assert_eq!(result.status, ExecutorResultStatus::Ok);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "Hako!");
        assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    }

    #[test]
    fn test_run_multiple_times() {
        let sandbox = sandbox();
        for _ in 0..256 {
            let mut executor = sandbox.command("/bin/true", &["true"]);
            let result = executor.run();
            assert_eq!(result.status, ExecutorResultStatus::Ok);
            assert_eq!(result.exit_code, Some(0));
        }
    }
}
