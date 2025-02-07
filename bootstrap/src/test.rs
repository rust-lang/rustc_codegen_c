use std::fs::File;
use std::path::{Path, PathBuf};

use anstream::{eprint as print, eprintln as println};
use clap::Args;
use color_print::{cprint, cprintln};
use glob::glob;
use similar::{ChangeTag, TextDiff};
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

        if manifest.verbose {
            cprintln!("<b>[TEST]</b> preparing to run tests with manifest: {:?}", manifest);
        }

        cprintln!("<b>[TEST]</b> running cargo test");
        let mut command = std::process::Command::new("cargo");
        command.args(["test", "--manifest-path", "crates/Cargo.toml"]);
        if manifest.verbose {
            cprintln!("<b>[TEST]</b> executing command: {:?}", command);
        }
        log::debug!("running {:?}", command);
        assert!(command.status().unwrap().success(), "failed to run {:?}", command);

        let testcases = self.collect_testcases(manifest);
        cprintln!("<b>[TEST]</b> found {} testcases", testcases.len());
        if manifest.verbose {
            for case in &testcases {
                cprintln!("<b>[TEST]</b> found test: {} ({:?})", case.name, case.test);
            }
        }

        let filechecker = FileChecker::new();
        for testcase in testcases {
            match testcase.test {
                TestType::FileCheck => {
                    if manifest.verbose {
                        cprintln!("<b>[TEST]</b> file checking <cyan>{}</cyan>", testcase.name);
                        cprintln!("       source: {}", testcase.source.display());
                        cprintln!("       output: {}", testcase.output_file.display());
                    } else {
                        cprint!("File checking {}... ", testcase.name);
                    }
                    testcase.build(manifest);
                    filechecker.run(&testcase);
                }
                TestType::Bless => {
                    if manifest.verbose {
                        cprintln!("<b>[TEST]</b> blessing <cyan>{}</cyan>", testcase.name);
                        cprintln!("       source: {}", testcase.source.display());
                        cprintln!("       output: {}", testcase.output_file.display());
                    } else {
                        cprint!("Blessing {}... ", testcase.name);
                    }
                    testcase.build(manifest);
                    bless(self.bless, &testcase);
                }
                TestType::Compile => {
                    if manifest.verbose {
                        cprintln!("<b>[TEST]</b> compiling <cyan>{}</cyan>", testcase.name);
                        cprintln!("       source: {}", testcase.source.display());
                        cprintln!("       output: {}", testcase.output_file.display());
                    } else {
                        cprint!("Compiling {}... ", testcase.name);
                    }
                    testcase.build(manifest);
                }
                TestType::CompileLib => {
                    if manifest.verbose {
                        cprintln!("<b>[TEST]</b> compiling lib <cyan>{}</cyan>", testcase.name);
                        cprintln!("       source: {}", testcase.source.display());
                        cprintln!("       output: {}", testcase.output_file.display());
                    } else {
                        cprint!("Compiling lib {}... ", testcase.name);
                    }
                    testcase.build_lib(manifest);
                }
            }
            if !manifest.verbose {
                cprintln!("<g>OK</g>");
            }
        }
    }
}

impl TestCommand {
    pub fn collect_testcases(&self, manifest: &Manifest) -> Vec<TestCase> {
        let mut tests = vec![];

        // Examples
        for case in glob("examples/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("examples/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("examples").join(filename);
            tests.push(TestCase { name, source: case, output_file, test: TestType::Compile })
        }

        // Codegen tests
        for case in glob("tests/codegen/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("codegen/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/codegen").join(filename);
            tests.push(TestCase { name, source: case, output_file, test: TestType::FileCheck })
        }

        // Bless tests - the output should be the same as the last run
        for case in glob("tests/bless/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("bless/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/bless").join(filename);
            tests.push(TestCase { name, source: case, output_file, test: TestType::Bless })
        }

        // Collect test-auxiliary
        let aux_use = regex::Regex::new(r"^//@\s*aux-build:(?P<fname>.*)").unwrap();
        let mut auxiliary = vec![];
        for case in tests.iter() {
            let source = std::fs::read_to_string(&case.source).unwrap();
            for cap in aux_use.captures_iter(&source) {
                let fname = cap.name("fname").unwrap().as_str();
                let source = Path::new("tests/auxiliary").join(fname);
                let filename = source.file_stem().unwrap();
                let name = format!("auxiliary/{}", filename.to_string_lossy());
                let output_file = manifest.out_dir.join(filename); // aux files are output to the base directory
                auxiliary.push(TestCase { name, source, output_file, test: TestType::CompileLib })
            }
        }

        // Compile auxiliary before the tests
        let mut cases = auxiliary;
        cases.extend(tests);
        cases
    }
}

#[derive(Debug)]
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
        if manifest.verbose {
            cprintln!("       command: {}", format!("{:?}", command).replace('"', ""));
        }
        log::debug!("running {:?}", command);
        let status = command.status().unwrap();
        if manifest.verbose {
            if status.success() {
                cprintln!("       <g>success</g>");
            }
        }
    }

    pub fn build_lib(&self, manifest: &Manifest) {
        let output_dir = self.output_file.parent().unwrap();
        std::fs::create_dir_all(output_dir).unwrap();
        let mut command = manifest.rustc();
        command
            .args(["--crate-type", "lib"])
            .arg("-O")
            .arg(&self.source)
            .arg("--out-dir")
            .arg(output_dir);
        if manifest.verbose {
            cprintln!("       command: {}", format!("{:?}", command).replace('"', ""));
        }
        log::debug!("running {:?}", command);
        let status = command.status().unwrap();
        if manifest.verbose {
            if status.success() {
                cprintln!("       <g>success</g>");
            }
        }
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
        let output = std::fs::read_to_string(output).unwrap();
        let blessed = std::fs::read_to_string(blessed).unwrap();

        let diff = TextDiff::from_lines(&blessed, &output);
        if diff.ratio() < 1.0 {
            cprintln!("<r,s>output does not match blessed output</r,s>");
            for change in diff.iter_all_changes() {
                let lineno = change.old_index().unwrap_or(change.new_index().unwrap_or(0));
                match change.tag() {
                    ChangeTag::Equal => print!(" {:4}| {}", lineno, change),
                    ChangeTag::Insert => cprint!("<g>+{:4}| {}</g>", lineno, change),
                    ChangeTag::Delete => cprint!("<r>-{:4}| {}</r>", lineno, change),
                }
            }
            std::process::exit(1);
        }
    }
}
