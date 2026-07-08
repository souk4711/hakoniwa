#[cfg(test)]
mod child_test {
    use std::{thread, time};

    use hakoniwa::{Command, Container, Runctl};

    fn command(program: &str) -> Command {
        Container::new().rootfs("/").unwrap().command(program)
    }

    fn command_as_tracee(program: &str) -> Command {
        Container::new()
            .runctl(Runctl::GetProcPidStatus)
            .rootfs("/")
            .unwrap()
            .command(program)
    }

    #[test]
    fn test_try_wait_terminated() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            let status = child.try_wait().unwrap();
            assert!(status.is_none());

            thread::sleep(time::Duration::from_secs(2));
            let status = child.try_wait().unwrap();
            assert!(status.is_some());
            assert!(status.unwrap().success());
        }
    }

    #[test]
    fn test_try_wait_timeout() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("4").wait_timeout(1).spawn().unwrap();

            let status = child.try_wait().unwrap();
            assert!(status.is_none());

            thread::sleep(time::Duration::from_secs(2));
            let status = child.try_wait().unwrap().unwrap();
            assert!(!status.success());
            assert_eq!(status.code, 128 + 9);
            assert_eq!(status.reason, "process(/bin/sleep) received signal SIGKILL");
            assert_eq!(status.exit_code, None);
            assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
        }
    }

    #[test]
    fn test_try_wait_killed() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            let status = child.try_wait().unwrap();
            assert!(status.is_none());

            child.kill().unwrap();
            thread::sleep(time::Duration::from_secs(2));
            let status = child.try_wait().unwrap().unwrap();
            assert!(!status.success());
            assert_eq!(status.code, 125);
            assert_eq!(status.reason, "container received signal SIGKILL");
            assert_eq!(status.exit_code, None);
            assert!(status.rusage.is_none());
        }
    }

    #[test]
    fn test_try_wait_then_wait() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            let status = child.try_wait().unwrap();
            assert!(status.is_none());

            let status = child.wait().unwrap();
            assert!(status.success());
        }
    }

    #[test]
    fn test_wait_terminated() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            thread::sleep(time::Duration::from_secs(2));
            let status = child.wait().unwrap();
            assert!(status.success());
        }
    }

    #[test]
    fn test_wait_timeout() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("4").wait_timeout(1).spawn().unwrap();

            thread::sleep(time::Duration::from_secs(2));
            let status = child.wait().unwrap();
            assert!(!status.success());
            assert_eq!(status.code, 128 + 9);
            assert_eq!(status.reason, "process(/bin/sleep) received signal SIGKILL");
            assert_eq!(status.exit_code, None);
            assert_eq!(status.rusage.unwrap().real_time.as_secs(), 1);
        }
    }

    #[test]
    fn test_wait_killed() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            child.kill().unwrap();
            let status = child.wait().unwrap();
            assert!(!status.success());
            assert_eq!(status.code, 125);
            assert_eq!(status.reason, "container received signal SIGKILL");
            assert_eq!(status.exit_code, None);
            assert!(status.rusage.is_none());
        }
    }

    #[test]
    fn test_wait_then_wait() {
        for mut cmd in vec![command("/bin/sleep"), command_as_tracee("/bin/sleep")] {
            let mut child = cmd.arg("1").spawn().unwrap();

            let status = child.wait().unwrap();
            assert!(status.success());

            let status = child.wait().unwrap();
            assert!(status.success());
        }
    }
}
