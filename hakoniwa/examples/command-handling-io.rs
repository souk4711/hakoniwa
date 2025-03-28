use hakoniwa::*;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container.rootfs("/");

    // spawn `echo` process
    let mut echo_child = container
        .command("/bin/echo")
        .arg("Oh no, a tpyo!")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // spawn `sed` process
    let mut sed_child = container
        .command("/bin/sed")
        .arg("s/tpyo/typo/")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // echo#stdout -> sed#stdin
    let mut echo_stdout = echo_child.stdout.take().unwrap();
    let mut sed_stdin = sed_child.stdin.take().unwrap();
    std::thread::spawn(move || {
        let mut data = vec![];
        echo_stdout.read_to_end(&mut data).unwrap();
        sed_stdin.write_all(&data).unwrap();
    })
    .join()
    .unwrap();

    // wait
    let output = sed_child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Oh no, a typo!\n");

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
