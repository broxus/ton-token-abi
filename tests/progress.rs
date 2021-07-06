#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/parser.rs");
    t.pass("tests/definition.rs");
}
