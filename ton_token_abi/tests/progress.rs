#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/enum.rs");
    t.pass("tests/names.rs");
    t.pass("tests/parse_with.rs");
    t.pass("tests/plain_struct.rs");
    t.pass("tests/struct.rs");
    t.pass("tests/types.rs");
}
