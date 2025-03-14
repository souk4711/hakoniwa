#[test]
fn example_test() {
    trycmd::TestCases::new()
        .case("examples/usage-*.md")
        .case("examples/howto-*.md");
}
