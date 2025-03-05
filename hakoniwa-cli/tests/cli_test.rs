#[test]
fn cli_test() {
    trycmd::TestCases::new()
        .case("tests/cli/unshare/*.md")
        .case("tests/cli/mount/*.md")
        .case("tests/cli/limit/*.md")
        .case("tests/cli/seccomp/*.md")
        .case("tests/cli/command/*.md")
        .case("tests/cli/misc/*.md");
}
