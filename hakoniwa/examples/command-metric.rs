use hakoniwa::{Container, Error};

fn main() -> Result<(), Error> {
    let mut container = Container::new();
    container.rootfs("/");

    let output = container.command("/bin/sleep").arg("2").output().unwrap();
    assert!(output.status.success());

    let rusage = output.status.rusage.unwrap();
    println!("   Real Time: {} sec", rusage.real_time.as_secs_f64());
    println!("   User Time: {} sec", rusage.user_time.as_secs_f64());
    println!(" System Time: {} sec", rusage.system_time.as_secs_f64());
    println!("Total Memory: {} kb", rusage.max_rss);

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
