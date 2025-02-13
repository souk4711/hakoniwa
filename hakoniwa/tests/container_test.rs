#[cfg(test)]
mod container_test {
    use hakoniwa::{Container, Namespace};

    #[test]
    #[ignore = "unshare(CloneFlags(CLONE_NEWUTS)) => EPERM: Operation not permitted"]
    fn test_hostname() {
        let mut container = Container::new();
        container
            .unshare_namespace(Namespace::Uts)
            .hostname("myhost");
        let mut command = container.command("/usr/bin/hostname");
        let output = command.output().unwrap();
        assert_eq!(output.status.success(), true);
        assert_eq!(String::from_utf8_lossy(&output.stdout), "myhost\n")
    }
}
