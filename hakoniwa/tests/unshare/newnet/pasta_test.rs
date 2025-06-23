use nix::unistd::Pid;

use hakoniwa::Pasta;

#[test]
pub fn test_args_default() {
    let pasta = Pasta::default();
    assert_eq!(
        pasta.to_cmdline(Pid::from_raw(0)),
        [
            "pasta",
            "--config-net",
            "--no-map-gw",
            "--tcp-ports",
            "none",
            "--udp-ports",
            "none",
            "--tcp-ns",
            "none",
            "--udp-ns",
            "none",
            "0"
        ]
    );
}

#[test]
pub fn test_args_map_gw() {
    let mut pasta = Pasta::default();
    pasta.args(["--map-gw"]);
    assert_eq!(
        pasta.to_cmdline(Pid::from_raw(1)),
        [
            "pasta",
            "--config-net",
            "--tcp-ports",
            "none",
            "--udp-ports",
            "none",
            "--tcp-ns",
            "none",
            "--udp-ns",
            "none",
            "1"
        ]
    );
}

#[test]
pub fn test_args_ports_options() {
    let mut pasta = Pasta::default();
    pasta.args([
        "--tcp-ports",
        "all",
        "--udp-ports",
        "auto",
        "--tcp-ns",
        "22:23",
        "--udp-ns",
        "192.0.2.1/22",
    ]);
    assert_eq!(
        pasta.to_cmdline(Pid::from_raw(2)),
        [
            "pasta",
            "--config-net",
            "--no-map-gw",
            "--tcp-ports",
            "all",
            "--udp-ports",
            "auto",
            "--tcp-ns",
            "22:23",
            "--udp-ns",
            "192.0.2.1/22",
            "2"
        ]
    );
}

#[test]
pub fn test_args_ports_options_alias() {
    let mut pasta = Pasta::default();
    pasta.args([
        "-t",
        "all",
        "-u",
        "auto",
        "-T",
        "22:23",
        "-U",
        "192.0.2.1/22",
    ]);
    assert_eq!(
        pasta.to_cmdline(Pid::from_raw(3)),
        [
            "pasta",
            "--config-net",
            "--no-map-gw",
            "-t",
            "all",
            "-u",
            "auto",
            "-T",
            "22:23",
            "-U",
            "192.0.2.1/22",
            "3"
        ]
    );
}
