#[derive(Clone, Debug)]
pub struct Pasta {
    prog: String,
    args: Vec<String>,
}

impl Pasta {
    pub fn program(&mut self, program: &str) -> &mut Self {
        self.prog = program.to_string();
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.args.push(arg.as_ref().to_string());
        }
        self
    }

    // [podman#createPastaArgs]: https://github.com/containers/common/blob/33bf9345b5efc6d43600e60f2a7b2a71cd9abdb5/libnetwork/pasta/pasta_linux.go#L164
    pub fn to_cmdline(&self, child: nix::unistd::Pid) -> Vec<String> {
        let mut no_map_gw = true;
        let mut no_tcp_ports = true;
        let mut no_udp_ports = true;
        let mut no_tcp_ns_ports = true;
        let mut no_udp_ns_ports = true;

        let mut cmdline = vec![];
        cmdline.push(self.prog.clone());
        cmdline.push("--config-net".to_string());

        let mut args = vec![];
        for arg in self.args.iter() {
            match arg.as_ref() {
                "--map-gw" => {
                    no_map_gw = false;
                    continue;
                }
                "-t" | "--tcp-ports" => no_tcp_ports = false,
                "-u" | "--udp-ports" => no_udp_ports = false,
                "-T" | "--tcp-ns" => no_tcp_ns_ports = false,
                "-U" | "--udp-ns" => no_udp_ns_ports = false,
                _ => {}
            };
            args.push(arg.to_string());
        }

        if no_map_gw {
            cmdline.push("--no-map-gw".to_string());
        }
        if no_tcp_ports {
            cmdline.push("--tcp-ports".to_string());
            cmdline.push("none".to_string());
        }
        if no_udp_ports {
            cmdline.push("--udp-ports".to_string());
            cmdline.push("none".to_string());
        }
        if no_tcp_ns_ports {
            cmdline.push("--tcp-ns".to_string());
            cmdline.push("none".to_string());
        }
        if no_udp_ns_ports {
            cmdline.push("--udp-ns".to_string());
            cmdline.push("none".to_string());
        }
        cmdline.append(&mut args);
        cmdline.push(format!("{}", child));
        cmdline
    }
}

impl Default for Pasta {
    fn default() -> Self {
        Self {
            prog: "pasta".to_string(),
            args: vec![],
        }
    }
}
