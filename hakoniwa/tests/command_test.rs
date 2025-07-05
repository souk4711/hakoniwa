#[cfg(test)]
mod command_test {
    use assertables::*;
    use std::collections::HashMap;
    use std::io::prelude::*;

    use hakoniwa::{Command, Container, Runctl, Stdio};

    fn command(program: &str) -> Command {
        Container::new().rootfs("/").command(program)
    }

    #[test]
    fn test_new() {
        let command = command("/bin/sh");
        assert_eq!(command.get_program(), "/bin/sh");
        assert_eq!(command.get_args().len(), 0);
    }

    #[test]
    fn test_arg() {
        let mut command = command("/bin/ls");
        command.arg("-C").arg("/path/to/repo");
        assert_eq!(command.get_program(), "/bin/ls");
        assert_eq!(command.get_args().to_vec(), ["-C", "/path/to/repo"]);
    }

    #[test]
    fn test_args() {
        let mut command = command("/bin/ls");
        command.args(["-l", "-a"]);
        assert_eq!(command.get_program(), "/bin/ls");
        assert_eq!(command.get_args().to_vec(), ["-l", "-a"]);
    }

    #[test]
    fn test_env() {
        let output = command("/bin/env").env("MYENV", "1").output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "MYENV=1\n");
    }

    #[test]
    fn test_envs() {
        let mut envs = HashMap::new();
        envs.insert("MYENV1", "1");
        envs.insert("MYENV2", "2");
        let output = command("/bin/env").envs(envs).output().unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "MYENV1=1\n");
        assert_contains!(String::from_utf8_lossy(&output.stdout), "MYENV2=2\n");
    }

    #[test]
    fn test_current_dir() {
        let output = command("/bin/pwd").current_dir("/bin").output().unwrap();
        assert!(output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stdout), "/bin\n");
    }

    #[test]
    fn test_wait_timeout() {
        let status = command("/bin/sleep")
            .arg("2")
            .wait_timeout(1)
            .status()
            .unwrap();
        assert!(!status.success());
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.reason, "process(/bin/sleep) received signal SIGKILL");
        assert_eq!(status.exit_code, None);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_spawn() {
        let mut child = command("/bin/true").spawn().unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_spawn_stdin_inherit() {
        if std::env::var("CI").is_ok() {
            eprintln!("test command_test::test_spawn_stdin_inherit ... skipped, CI");
            return;
        }

        let mut child = command("/bin/wc").wait_timeout(1).spawn().unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.reason, "process(/bin/wc) received signal SIGKILL");
        assert_eq!(status.exit_code, None);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_spawn_stdin_piped() {
        let mut child = command("/bin/wc")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        std::thread::spawn(move || {
            stdin.write_all(b"stdin piped").unwrap();
        });
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "11\n");
    }

    #[test]
    fn test_spawn_stdout_inherit() {
        let mut child = command("bin/echo").arg("stdout inherit").spawn().unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        // "stdout inherit" echoed to console
    }

    #[test]
    fn test_spawn_stdout_piped() {
        let mut child = command("/bin/echo")
            .arg("stdout piped")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "stdout piped\n");
    }

    #[test]
    fn test_status_exit_code_zero() {
        let status = command("/bin/true").status().unwrap();
        assert!(status.success());
        assert_eq!(status.exit_code.unwrap(), 0);
    }

    #[test]
    fn test_status_exit_code_nonzero() {
        let status = command("/bin/false").status().unwrap();
        assert!(!status.success());
        assert_eq!(status.code, 1);
        assert_eq!(status.exit_code.unwrap(), 1);
    }

    #[test]
    fn test_status_rusage() {
        let status = command("/bin/sleep").arg("1").status().unwrap();
        assert!(status.success());

        let r = status.rusage.unwrap();
        assert_eq!(r.real_time.as_secs(), 1);

        let r = status.proc_pid_status;
        assert!(r.is_none());

        let r = status.proc_pid_smaps_rollup;
        assert!(r.is_none());
    }

    #[test]
    fn test_status_proc_pid_smaps_rollup() {
        let _100mb: Vec<u8> = vec![10; 1024 * 1024 * 100];
        let status = Container::new()
            .runctl(Runctl::GetProcPidSmapsRollup)
            .rootfs("/")
            .command("/bin/ls")
            .status()
            .unwrap();
        assert!(status.success());

        let r = status.rusage.unwrap();
        assert!(r.max_rss > 1024 * 100);

        let r = status.proc_pid_status;
        assert!(r.is_none());

        let r = status.proc_pid_smaps_rollup.unwrap();
        let shared = r.shared_clean + r.shared_dirty;
        let private = r.private_clean + r.private_dirty;
        assert_eq!(r.rss, shared + private);
        assert_eq!(r.pss, r.pss_anon + r.pss_file + r.pss_shmem);
        assert!(r.rss < 1024 * 10);
        assert!(r.pss < 1024 * 10);
    }

    #[test]
    fn test_status_proc_pid_status() {
        let _100mb: Vec<u8> = vec![10; 1024 * 1024 * 100];
        let status = Container::new()
            .runctl(Runctl::GetProcPidStatus)
            .rootfs("/")
            .command("/bin/ls")
            .status()
            .unwrap();
        assert!(status.success());

        let r = status.rusage.unwrap();
        assert!(r.max_rss > 1024 * 100);

        let r = status.proc_pid_status.unwrap();
        assert_eq!(r.vmrss, r.rssanon + r.rssfile + r.rssshmem);
        assert!(r.vmrss < 1024 * 10);
        assert!(r.vmhwm < 1024 * 10);

        let r = status.proc_pid_smaps_rollup;
        assert!(r.is_none());
    }

    #[test]
    fn test_output_stdout_piped() {
        let output = command("/bin/echo").arg("Hello, World!").output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, World!\n");
    }

    #[test]
    fn test_output_stdout_inherit() {
        let output = command("/bin/echo")
            .arg("Hello, World!")
            .stdout(Stdio::inherit())
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        // "Hello, World!" echoed to console
    }

    #[test]
    fn test_output_stderr_piped() {
        let output = command("/bin/grep").output().unwrap();
        assert!(!output.status.success());
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Usage: ");
    }

    #[test]
    fn test_output_stderr_inherit() {
        let output = command("/bin/grep")
            .stderr(Stdio::inherit())
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
        // error message echoed to console
    }

    #[test]
    fn test_write_error_broken_pipe() {
        let mut child = Container::new()
            .rootfs("/")
            .devfsmount("/dev")
            .command("/bin/bash")
            .args(["-c", "cat /dev/random | base64 | head -n1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");

        let mut child = Container::new()
            .rootfs("/")
            .devfsmount("/dev")
            .command("/bin/bash")
            .args(["-c", "set -o pipefail; cat /dev/random | base64 | head -n1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(!output.status.success());
        assert_eq!(output.status.code, 141);
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }

    #[test]
    fn test_stderr_writer_donot_blocked_indefinitely() {
        let output = Container::new()
            .rootfs("/")
            .devfsmount("/dev")
            .command("/bin/bash")
            .args([
                "-c",
                "dd if=/dev/urandom of=/dev/stdout bs=1M count=1 status=none",
            ])
            .wait_timeout(1)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 1024 * 1024);

        let output = Container::new()
            .rootfs("/")
            .devfsmount("/dev")
            .command("/bin/bash")
            .args([
                "-c",
                "dd if=/dev/urandom of=/dev/stderr bs=1M count=1 status=none",
            ])
            .wait_timeout(1)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stderr.len(), 1024 * 1024);
    }
}
