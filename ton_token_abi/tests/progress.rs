#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/common.rs");
    t.pass("tests/names.rs");
    t.pass("tests/types.rs");
}
