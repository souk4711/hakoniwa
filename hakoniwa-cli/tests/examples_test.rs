#[test]
fn example_test() {
    trycmd::TestCases::new()
        .case("examples/usage-unshare.md")
        .case("examples/usage-mount.md")
        .case("examples/usage-limit.md")
        .case("examples/usage-command.md")
        .case("examples/usage-misc.md");
}
