#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/vertex-parse.rs");
    t.pass("tests/vertex-valid.rs");
    t.pass("tests/vertex-arrays.rs");
}