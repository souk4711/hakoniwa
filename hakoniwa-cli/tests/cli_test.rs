#[test]
fn cli_test() {
    let t = trycmd::TestCases::new();
    t.case("tests/cli/*/*.md");
    #[cfg(not(feature = "__cgroups"))]
    t.skip("tests/cli/cgroups/*.md");
}
