use std::fs::File;
use std::path::PathBuf;

use anstream::{eprint as print, eprintln as println};
use clap::Args;
use color_print::{cprint, cprintln};
use glob::glob;
use which::which;

use crate::manifest::Manifest;
use crate::Run;

/// Run tests
#[derive(Args, Debug)]
pub struct TestCommand {
    /// Update the blessed output
    #[clap(long)]
    pub bless: bool,
}

impl Run for TestCommand {
    fn run(&self, manifest: &Manifest) {
        manifest.prepare();

        std::panic::set_hook(Box::new(|info| {
            cprintln!("<r,s>Test failed</r,s>: {}", info);
        }));

        cprintln!("<b>[TEST]</b> running cargo test");
        let mut command = std::process::Command::new("cargo");
        command.args(["test", "--manifest-path", "crates/Cargo.toml"]);
        log::debug!("running {:?}", command);
        assert!(command.status().unwrap().success(), "failed to run {:?}", command);

        let testcases = self.collect_testcases(manifest);
        cprintln!("<b>[TEST]</b> found {} testcases", testcases.len());

        let filechecker = FileChecker::new();
        for testcase in testcases {
            match testcase.test {
                TestType::FileCheck => {
                    cprint!("File checking {}...", testcase.name);
                    testcase.build(manifest);
                    filechecker.run(&testcase);
                }
                TestType::Bless => {
                    cprint!("Blessing {}...", testcase.name);
                    testcase.build(manifest);
                    bless(self.bless, &testcase);
                }
                TestType::Compile => {
                    cprint!("Compiling {}...", testcase.name);
                    testcase.build(manifest);
                }
                TestType::CompileLib => {
                    cprint!("Compiling lib {}...", testcase.name);
                    testcase.build_lib(manifest);
                }
            }
            cprintln!("<g>OK</g>");
        }
    }
}

impl TestCommand {
    pub fn collect_testcases(&self, manifest: &Manifest) -> Vec<TestCase> {
        let mut result = vec![];

        // Test auxiliary (should compile first)
        for case in glob("tests/auxiliary/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("auxiliary/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join(filename);
            result.push(TestCase { name, source: case, output_file, test: TestType::CompileLib })
        }

        // Examples
        for case in glob("examples/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("examples/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("examples").join(filename);
            result.push(TestCase { name, source: case, output_file, test: TestType::Compile })
        }

        // Codegen tests
        for case in glob("tests/codegen/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("codegen/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/codegen").join(filename);
            result.push(TestCase { name, source: case, output_file, test: TestType::FileCheck })
        }

        // Bless tests - the output should be the same as the last run
        for case in glob("tests/bless/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("bless/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/bless").join(filename);
            result.push(TestCase { name, source: case, output_file, test: TestType::Bless })
        }

        result
    }
}

pub enum TestType {
    /// Test an executable can be compiled
    Compile,
    /// Test a library can be compiled
    CompileLib,
    /// Run LLVM FileCheck on the generated code
    FileCheck,
    /// Bless test - the output should be the same as the last run
    Bless,
}

pub struct TestCase {
    pub name: String,
    pub source: PathBuf,
    pub output_file: PathBuf,
    pub test: TestType,
}

impl TestCase {
    pub fn build(&self, manifest: &Manifest) {
        let output_dir = self.output_file.parent().unwrap();
        std::fs::create_dir_all(output_dir).unwrap();
        let mut command = manifest.rustc();
        command
            .args(["--crate-type", "bin"])
            .arg("-O")
            .arg(&self.source)
            .arg("-o")
            .arg(&self.output_file);
        log::debug!("running {:?}", command);
        command.status().unwrap();
    }

    pub fn build_lib(&self, manifest: &Manifest) {
        let output_dir = self.output_file.parent().unwrap();
        std::fs::create_dir_all(output_dir).unwrap();
        let mut command = manifest.rustc();
        command
            .args(["--crate-type", "lib"])
            .arg("-O")
            .arg(&self.source)
            .arg("--out-dir") // we use `--out-dir` to integrate with the default name convention
            .arg(output_dir); // so here we ignore the filename and just use the directory
        log::debug!("running {:?}", command);
        command.status().unwrap();
    }

    /// Get the generated C file f
    pub fn generated(&self) -> PathBuf {
        let case = self.source.file_stem().unwrap().to_string_lossy();
        let generated = std::fs::read_dir(self.output_file.parent().unwrap())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let filename = entry.file_name();
                let filename = filename.to_string_lossy();
                filename.ends_with(".c") && filename.starts_with(case.as_ref())
            });

        assert!(generated.is_some(), "could not find {case}'s generated file");
        generated.unwrap().path()
    }
}

struct FileChecker {
    filecheck: PathBuf,
}

impl FileChecker {
    pub fn new() -> Self {
        let filecheck = [
            "FileCheck-18",
            "FileCheck-17",
            "FileCheck-16",
            "FileCheck-15",
            "FileCheck-14",
            "FileCheck",
        ]
        .into_iter()
        .find_map(|filecheck| which(filecheck).ok())
        .expect("`FileCheck` not found");

        Self { filecheck }
    }

    fn run(&self, case: &TestCase) {
        let generated = File::open(case.generated()).unwrap();
        let mut command = std::process::Command::new(&self.filecheck);
        command.arg(&case.source).stdin(generated);
        log::debug!("running {:?}", command);
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "failed to run FileCheck on {}",
            case.source.file_stem().unwrap().to_string_lossy()
        );
    }
}

fn bless(update: bool, case: &TestCase) {
    let output = case.generated();
    let blessed = case.source.with_extension("c");
    if update {
        std::fs::copy(output, blessed).unwrap();
    } else {
        let output = std::fs::read(output).unwrap();
        let blessed = std::fs::read(blessed).unwrap();
        assert_eq!(output, blessed, "output does not match blessed output");
    }
}
