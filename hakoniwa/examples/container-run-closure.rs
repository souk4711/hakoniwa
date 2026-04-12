use hakoniwa::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container.rootfs("/")?;

    let output = container.run(|ctx| {
        println!("running inside container, root: {:?}", ctx.root);
        (0, 42u32)
    })?;

    assert!(output.status.success());
    assert_eq!(output.data, 42);
    println!("closure returned: {}", output.data);
    println!("exit status: {}", output.status.reason);

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
