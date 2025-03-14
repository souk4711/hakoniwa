#[test]
fn cli_test() {
    trycmd::TestCases::new().case("tests/cli/*/*.md");
}
