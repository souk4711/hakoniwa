#[cfg(test)]
mod container_test {
    use hakoniwa::{Container, Namespace};

    #[test]
    fn test_hostname() {
        let mut container = Container::new();
        container
            .unshare_namespace(Namespace::User) // EPERM
            .unshare_namespace(Namespace::Uts)
            .hostname("myhost");
        let mut command = container.command("/bin/hostname");
        let output = command.output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "myhost\n")
    }
}
