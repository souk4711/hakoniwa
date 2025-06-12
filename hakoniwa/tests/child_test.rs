#[cfg(test)]
mod child_test {
    use std::{thread, time};

    use hakoniwa::{Command, Container};

    fn command(program: &str) -> Command {
        Container::new().rootfs("/").command(program)
    }

    #[test]
    fn test_try_wait_terminated() {
        let mut child = command("/bin/sleep").arg("1").spawn().unwrap();

        // running
        let status = child.try_wait().unwrap();
        assert!(status.is_none());

        // child has finished its execution
        thread::sleep(time::Duration::from_secs(2));
        let status = child.try_wait().unwrap();
        assert!(status.is_some());
        assert!(status.unwrap().success());
    }

    #[test]
    fn test_try_wait_killed() {
        let mut child = command("/bin/sleep")
            .arg("4")
            .wait_timeout(1)
            .spawn()
            .unwrap();

        // running
        let status = child.try_wait().unwrap();
        assert!(status.is_none());

        // child was killed after the timeout period
        thread::sleep(time::Duration::from_secs(2));
        let status = child.try_wait().unwrap().unwrap();
        assert!(!status.success());
        assert_eq!(status.code, 128 + 9);
        assert_eq!(status.reason, "Process(/bin/sleep) received signal SIGKILL");
        assert_eq!(status.exit_code, None);
        assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
    }

    #[test]
    fn test_try_wait_then_wait() {
        let mut child = command("/bin/sleep").arg("1").spawn().unwrap();

        // running
        let status = child.try_wait().unwrap();
        assert!(status.is_none());

        // status not ready yet, let's really wait
        let status = child.wait().unwrap();
        assert!(status.success());
    }
}
