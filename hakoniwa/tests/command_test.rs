#[cfg(test)]
mod command_test {
    use assertables::assert_contains;
    use std::io::prelude::*;

    use hakoniwa::{Command, Container, Stdio};

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
        command.args(&["-l", "-a"]);
        assert_eq!(command.get_program(), "/bin/ls");
        assert_eq!(command.get_args().to_vec(), ["-l", "-a"]);
    }

    #[test]
    fn test_wait_timeout() {
        let mut command = command("/bin/sleep");
        let status = command.arg("2").wait_timeout(1).status().unwrap();
        assert_eq!(status.success(), false);
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.reason, "waitpid(...) => Signaled(_, SIGKILL, _)");
        assert_eq!(status.exit_code, None);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_spawn() {
        let mut command = command("/bin/true");
        let mut child = command.spawn().unwrap();
        let status = child.wait().unwrap();
        assert_eq!(status.success(), true);
    }

    #[test]
    fn test_spawn_stdin_inherit() {
        let mut command = command("/bin/wc");
        let mut child = command.wait_timeout(1).spawn().unwrap();
        let status = child.wait().unwrap();
        assert_eq!(status.success(), false);
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_spawn_stdin_piped() {
        let mut command = command("/bin/wc");
        let mut child = command
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
        let status = output.status;
        assert_eq!(status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "11\n");
    }

    #[test]
    fn test_spawn_stdout_inherit() {
        let mut command = command("/bin/echo");
        let mut child = command.arg("stdout inherit").spawn().unwrap();
        let output = child.wait_with_output().unwrap();
        let status = output.status;
        assert_eq!(status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        // "stdout inherit" echoed to console
    }

    #[test]
    fn test_spawn_stdout_piped() {
        let mut command = command("/bin/echo");
        let mut child = command
            .arg("stdout piped")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        let status = output.status;
        assert_eq!(status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "stdout piped\n");
    }

    #[test]
    fn test_status_exit_code_zero() {
        let mut command = command("/bin/true");
        let status = command.status().unwrap();
        assert_eq!(status.success(), true);
        assert_eq!(status.exit_code.unwrap(), 0);
    }

    #[test]
    fn test_status_exit_code_nonzero() {
        let mut command = command("/bin/false");
        let status = command.status().unwrap();
        assert_eq!(status.success(), false);
        assert_eq!(status.code, 1);
        assert_eq!(status.exit_code.unwrap(), 1);
    }

    #[test]
    fn test_status_rusage() {
        let mut command = command("/bin/sleep");
        let status = command.arg("1").status().unwrap();
        assert_eq!(status.success(), true);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_output_stdout() {
        let mut command = command("/bin/echo");
        let output = command.arg("Hello, World!").output().unwrap();
        let status = output.status;
        assert_eq!(status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, World!\n");
    }

    #[test]
    fn test_output_stderr() {
        let mut command = command("/bin/grep");
        let output = command.output().unwrap();
        let status = output.status;
        assert_eq!(status.success(), false);
        assert_contains!(String::from_utf8_lossy(&output.stderr), "Usage: ");
    }
}
