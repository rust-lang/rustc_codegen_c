use std::path::Path;

use rustc_codegen_c_ast::pretty::{Print, PrinterCtx};
use rustc_codegen_c_ast::{ModuleArena, ModuleCtx};

/// Run a blessed test.
///
/// When the environment variable `RUST_BLESS` is set, this function will store
/// the output of the test case in the corresponding file, otherwise it will
/// compare the output of the test case with the stored output, making sure they
/// are the same.
#[track_caller]
pub fn blessed_test(name: &str, bless: impl Fn() -> String) {
    let test_case_path = Path::new("tests/blessed").join(name).with_extension("out");
    test_case_path.parent().map(std::fs::create_dir_all);

    let output = bless();
    let expected_output = std::fs::read_to_string(&test_case_path).unwrap_or_default();
    if std::env::var("RUST_BLESS").is_ok() {
        std::fs::write(test_case_path, output).unwrap();
    } else {
        assert_eq!(output, expected_output, "blessed test '{name}' failed");
    }
}

/// Run a blessed test for a printable value.
pub fn printer_test<F>(name: &str, test: F)
where
    F: for<'mx> Fn(ModuleCtx<'mx>) -> Box<dyn Print + 'mx>, // anyway to avoid the Box?
{
    blessed_test(name, || {
        let module = ModuleArena::new("// blessed test");
        let ctx = ModuleCtx(&module);

        let mut pp = PrinterCtx::new();
        test(ctx).print_to(&mut pp);
        pp.finish()
    });
}
