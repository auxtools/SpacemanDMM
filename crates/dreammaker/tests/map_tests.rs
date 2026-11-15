extern crate dreammaker as dm;

use std::path::PathBuf;

use dm::Preprocessor;

#[test]
fn maps_in_include_order() {
    let context = dm::Context::default();
    let mut preprocessor = Preprocessor::from_buffer(
        &context,
        "tests/root.dme".into(),
        "#include \"maps/a.dmm\"\n#ifdef SMALL\n#include \"maps/c.dmm\"\n#endif\n#include \"maps/b.dmm\"\n",
    );
    preprocessor.by_ref().count();
    context.assert_success();
    assert_eq!(
        preprocessor.maps(),
        [
            PathBuf::from("tests/maps/a.dmm"),
            PathBuf::from("tests/maps/b.dmm"),
        ],
    );
}
