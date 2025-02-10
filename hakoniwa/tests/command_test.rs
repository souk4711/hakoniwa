#[cfg(test)]
mod command_test {
    use hakoniwa::Container;

    #[test]
    fn test_new() {
        let container = Container::new();
        let command = container.command("/bin/sh");
        assert_eq!(command.get_program(), "/bin/sh");
        assert_eq!(command.get_args().len(), 0);
    }

    #[test]
    fn test_arg() {
        let container = Container::new();
        let mut command = container.command("/bin/ls");
        command.arg("-C").arg("/path/to/repo");
        assert_eq!(command.get_program(), "/bin/ls");
        assert_eq!(command.get_args().to_vec(), ["-C", "/path/to/repo"]);
    }

    #[test]
    fn test_args() {
        let container = Container::new();
        let mut command = container.command("/bin/ls");
        command.args(&["-l", "-a"]);
        assert_eq!(command.get_program(), "/bin/ls");
        assert_eq!(command.get_args().to_vec(), ["-l", "-a"]);
    }

    #[test]
    fn test_spawn() {
        let container = Container::new();
        let mut command = container.command("/bin/true");
        let mut child = command.spawn().unwrap();
        let status = child.wait().unwrap();
        assert_eq!(status.success(), true);
    }

    #[test]
    fn test_status_exit_code_zero() {
        let container = Container::new();
        let mut command = container.command("/bin/true");
        let status = command.status().unwrap();
        assert_eq!(status.success(), true);
        assert_eq!(status.exit_code, Some(0));
    }

    #[test]
    fn test_status_exit_code_nonzero() {
        let container = Container::new();
        let mut command = container.command("/bin/false");
        let status = command.status().unwrap();
        assert_eq!(status.success(), true);
        assert_eq!(status.exit_code, Some(1));
    }

    #[test]
    fn test_status_rusage() {
        let container = Container::new();
        let mut command = container.command("/bin/sleep");
        let status = command.arg("1").status().unwrap();
        assert_eq!(status.success(), true);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }
}
