use std::fs::File;
use std::path::{Path, PathBuf};

use anstream::{eprint as print, eprintln as println};
use clap::Args;
use color_print::{cprint, cprintln};
use glob::glob;
use which::which;

use crate::{manifest::Manifest, Run};

/// Run tests
#[derive(Args, Debug)]
pub struct TestCommand {}

impl Run for TestCommand {
    fn run(&self, manifest: &Manifest) {
        manifest.prepare();

        std::panic::set_hook(Box::new(|info| {
            cprintln!("<r,s>Test failed</r,s>: {}", info);
        }));

        let testcases = self.collect_testcases(manifest);
        cprintln!("<b>[TEST]</b> found {} testcases", testcases.len());

        let filechecker = FileChecker::new();
        for testcase in testcases {
            match testcase.test {
                TestType::FileCheck => {
                    cprint!("File checking {}...", testcase.name);
                    testcase.build(manifest);
                    filechecker.run(&testcase.source, &testcase.output);
                }
                TestType::Compile => {
                    cprint!("Compiling {}...", testcase.name);
                    testcase.build(manifest);
                }
            }
            cprintln!("<g>OK</g>");
        }
    }
}

impl TestCommand {
    pub fn collect_testcases(&self, manifest: &Manifest) -> Vec<TestCase> {
        let mut result = vec![];

        // Examples
        for case in glob("example/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            if filename == "mini_core" {
                continue;
            }
            let name = format!("example/{}", filename.to_string_lossy());
            let output = manifest.out_dir.join("example").join(filename);
            result.push(TestCase {
                name,
                source: case,
                output,
                test: TestType::Compile,
            })
        }

        // Codegen tests
        for case in glob("tests/codegen/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("codegen/{}", filename.to_string_lossy());
            let output = manifest.out_dir.join("tests/codegen").join(filename);
            result.push(TestCase {
                name,
                source: case,
                output,
                test: TestType::FileCheck,
            })
        }

        result
    }
}

pub enum TestType {
    Compile,
    FileCheck,
}

pub struct TestCase {
    pub name: String,
    pub source: PathBuf,
    pub output: PathBuf,
    pub test: TestType,
}

impl TestCase {
    pub fn build(&self, manifest: &Manifest) {
        std::fs::create_dir_all(self.output.parent().unwrap()).unwrap();
        let mut command = manifest.rustc();
        command
            .args(["--crate-type", "bin"])
            .arg("-O")
            .arg(&self.source)
            .arg("-o")
            .arg(&self.output);
        log::debug!("running {:?}", command);
        command.status().unwrap();
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

    fn run(&self, source: &Path, output: &Path) {
        let case = source.file_stem().unwrap().to_string_lossy();
        let generated = std::fs::read_dir(output.parent().unwrap())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let filename = entry.file_name();
                let filename = filename.to_string_lossy();
                filename.ends_with(".c") && filename.starts_with(case.as_ref())
            });

        assert!(
            generated.is_some(),
            "could not find {case}'s generated file"
        );
        let generated = generated.unwrap();

        let generated = File::open(generated.path()).unwrap();
        let mut command = std::process::Command::new(&self.filecheck);
        command.arg(source).stdin(generated);
        log::debug!("running {:?}", command);
        let output = command.output().unwrap();
        assert!(output.status.success(), "failed to run FileCheck on {case}");
    }
}
