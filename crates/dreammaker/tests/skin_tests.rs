extern crate dreammaker as dm;

use std::path::PathBuf;

use dm::Preprocessor;

#[test]
fn skins_in_include_order() {
    let context = dm::Context::default();
    let mut preprocessor = Preprocessor::from_buffer(
        &context,
        "tests/root.dme".into(),
        "#include \"skins/a.dmf\"\n#if 0\n#include \"skins/c.dmf\"\n#endif\n#include \"skins/b.dmf\"\n",
    );
    preprocessor.by_ref().count();
    context.assert_success();
    assert_eq!(
        preprocessor.skins(),
        [
            PathBuf::from("tests/skins/a.dmf"),
            PathBuf::from("tests/skins/b.dmf"),
        ],
    );
}
