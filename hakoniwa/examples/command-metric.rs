use hakoniwa::*;

fn main() -> Result<()> {
    let _100mb: Vec<u8> = vec![10; 1024 * 1024 * 100];

    let mut container = Container::new();
    container.rootfs("/")?;
    container.runctl(Runctl::GetProcPidStatus);
    container.runctl(Runctl::GetProcPidSmapsRollup);

    let output = container.command("/bin/sleep").arg("2").output().unwrap();
    assert!(output.status.success());

    let r = output.status.rusage.unwrap();
    println!("Rusage:");
    println!("   Real Time: {} sec", r.real_time.as_secs_f64());
    println!("   User Time: {} sec", r.user_time.as_secs_f64());
    println!(" System Time: {} sec", r.system_time.as_secs_f64());
    println!("     Max RSS: {} kB ", r.max_rss);

    let r = output.status.proc_pid_status.unwrap();
    println!();
    println!("ProcPidStatus (at exit):");
    println!("      VmPeak: {} kB ", r.vmpeak);
    println!("       VmHWM: {} kB ", r.vmhwm);
    println!("       VmRSS: {} kB ", r.vmrss);

    let r = output.status.proc_pid_smaps_rollup.unwrap();
    println!();
    println!("ProcPidSmapsRollup (at exit):");
    println!("         RSS: {} kB ", r.rss);
    println!("         PSS: {} kB ", r.pss);

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
