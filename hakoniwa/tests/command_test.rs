#[cfg(test)]
mod command_test {
    use hakoniwa::{Command, Container};

    fn command(program: &str) -> Command {
        Container::new().command(program)
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
        assert!(!status.success());
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.exit_code, None);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_spawn() {
        let mut command = command("/bin/true");
        let mut child = command.spawn().unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_status_exit_code_zero() {
        let mut command = command("/bin/true");
        let status = command.status().unwrap();
        assert!(status.success());
        assert_eq!(status.exit_code, Some(0));
    }

    #[test]
    fn test_status_exit_code_nonzero() {
        let mut command = command("/bin/false");
        let status = command.status().unwrap();
        assert!(status.success());
        assert_eq!(status.exit_code, Some(1));
    }

    #[test]
    fn test_status_rusage() {
        let mut command = command("/bin/sleep");
        let status = command.arg("1").status().unwrap();
        assert!(status.success());
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_output_stdout() {
        let mut command = command("/bin/echo");
        let output = command.arg("Hello, World!").output().unwrap();
        let status = output.status;
        assert!(status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, World!\n");
    }

    #[test]
    fn test_output_stderr() {
        let mut command = command("/bin/ping");
        let output = command.arg("invalid-host-name").output().unwrap();
        let status = output.status;
        assert!(status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Name or service not known"));
    }
}
