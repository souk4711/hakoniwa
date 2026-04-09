#[cfg(feature = "cgroups")]
fn main() -> Result<(), hakoniwa::Error> {
    use hakoniwa::{cgroups::*, *};

    let mut resources = Resources::default();
    let mut cpu = Cpu::default();
    let mut memory = Memory::default();
    let mut pids = Pids::default();

    let tim = 100_000;
    let mem = 512 * 1024 * 1024;

    #[rustfmt::skip]
    {
        cpu.weight(200);                       // CPUWeight=200
        cpu.quota(2 * tim).period(tim as u64); // CPUQuota=200%
        memory.limit(mem);                     // MemoryMax=512MiB
        memory.swap(mem);                      // MemorySwapMax=0
        pids.limit(4);                         // TasksMax=4
        resources.cpu(cpu).memory(memory).pids(pids);
    };

    let mut container = Container::new();
    container
        .rootfs("/")?
        .devfsmount("/dev")
        .tmpfsmount("/tmp")
        .cgroups_resources(resources);

    let status = container
        .command("/bin/dd")
        .args(["if=/dev/random", "of=/tmp/output.txt", "count=1", "bs=4"])
        .status()?;
    assert!(status.success());

    Ok(())
}

#[cfg(not(feature = "cgroups"))]
fn main() -> Result<(), hakoniwa::Error> {
    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
