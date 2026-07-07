use hakoniwa::*;

const PYTHON_CODE: &str = "x = 0
while True:
    x += 1";

fn main() -> Result<()> {
    let mut container = Container::new();
    container
        .rootfs("/")?
        .bindmount_ro("/usr", "/usr")
        .devfsmount("/dev")
        .tmpfsmount("/tmp")
        .setrlimit(Rlimit::As, 16_000_000, 16_000_000); // 16MB

    let mut command = container.command("/usr/bin/python3");
    command.arg("-c");
    command.arg(PYTHON_CODE);
    command.wait_timeout(2); // 2 seconds

    let status = command.status()?;
    // Process was killed by the timeout — code should be 128 + SIGKILL = 137
    // and metrics should be populated because the fix captures them
    // before the kill signal is delivered.
    assert!(!status.success());
    assert_eq!(status.code, 128 + libc::SIGKILL);

    // These would have been null before the fix
    assert!(status.rusage.is_some(), "rusage should be populated");
    assert!(
        status.proc_pid_status.is_some(),
        "proc_pid_status should be populated (was null before fix)"
    );
    assert!(
        status.proc_pid_smaps_rollup.is_some(),
        "proc_pid_smaps_rollup should be populated (was null before fix)"
    );

    // Print a summary of the captured metrics
    if let Some(r) = &status.rusage {
        println!("Wall time: {:.3}s", r.real_time.as_secs_f64());
        println!("User time: {:.3}s", r.user_time.as_secs_f64());
        println!("Max RSS:   {} kB", r.max_rss);
    }
    if let Some(s) = &status.proc_pid_status {
        println!("VmPeak:    {} kB", s.vmpeak);
        println!("VmHWM:     {} kB", s.vmhwm);
        println!("VmRSS:     {} kB", s.vmrss);
        println!("RssAnon:   {} kB", s.rssanon);
    }
    if let Some(s) = &status.proc_pid_smaps_rollup {
        println!("PSS:       {} kB", s.pss);
        println!("PrivDirty: {} kB", s.private_dirty);
    }

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}