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
    #[arg(long)]
    pub bless: bool,

    /// Whether to show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl Run for TestCommand {
    const STEP_DISPLAY_NAME: &'static str = "TEST";

    fn run(&self, manifest: &Manifest) {
        manifest.prepare();

        std::panic::set_hook(Box::new(|info| {
            cprintln!("<r,s>Test failed</r,s>: {}", info);
        }));

        // action: Run cargo test
        self.log_action_start("running", "cargo test");
        let mut command = std::process::Command::new("cargo");
        command.args(["test", "--manifest-path", "crates/Cargo.toml"]);
        self.command_status("cargo", &mut command);

        let testcases = self.collect_testcases(manifest);
        self.log_action_start(&format!("found {} testcases", testcases.len()), "");
        testcases.iter().for_each(|t| self.log_action_context(t.test.as_str(), t.name.as_str()));

        let filechecker = FileChecker::new(self.verbose);
        for testcase in testcases {
            match testcase.test {
                TestType::FileCheck => {
                    self.log_action_start("TEST file checking", &testcase.name);
                    self.log_action_context("source", &testcase.source.display());
                    self.log_action_context("output", &testcase.output_file.display());
                    testcase.build(manifest);
                    filechecker.check_testcase(&testcase);
                }
                TestType::Bless => {
                    self.log_action_start("TEST Bless", &testcase.name);
                    self.log_action_context("source", &testcase.source.display());
                    self.log_action_context("output", &testcase.output_file.display());
                    testcase.build(manifest);
                    self.bless(self.bless, &testcase);
                }
                TestType::Compile => {
                    self.log_action_start("TEST Compile", &testcase.name);
                    self.log_action_context("source", &testcase.source.display());
                    self.log_action_context("output", &testcase.output_file.display());
                    testcase.build(manifest);
                }
                TestType::CompileLib => {
                    self.log_action_start("TEST CompileLib", &testcase.name);
                    self.log_action_context("source", &testcase.source.display());
                    self.log_action_context("output", &testcase.output_file.display());
                    testcase.build_lib(manifest);
                }
                TestType::Runtime => {
                    self.log_action_start("TEST Runtime", &testcase.name);
                    self.log_action_context("source", &testcase.source.display());
                    self.log_action_context("output", &testcase.output_file.display());
                    testcase.build(manifest);
                    self.run_and_check_output(&testcase);
                }
            }
        }
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}

impl TestCommand {
    pub fn collect_testcases(&self, manifest: &Manifest) -> Vec<TestCase> {
        let mut cases = vec![];
        let verbose = self.verbose;

        // Examples
        for case in glob("examples/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("examples/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("examples").join(filename);
            let testcase = TestCase::new(name, case, output_file, TestType::Compile, verbose);
            cases.push(testcase);
        }

        // Codegen tests
        for case in glob("tests/codegen/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("codegen/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/codegen").join(filename);
            let testcase = TestCase::new(name, case, output_file, TestType::FileCheck, verbose);
            cases.push(testcase);
        }

        // Bless tests - the output should be the same as the last run
        for case in glob("tests/bless/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("bless/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/bless").join(filename);
            let testcase = TestCase::new(name, case, output_file, TestType::Bless, verbose);
            cases.push(testcase);
        }

        // Runtime tests - compile, run and compare output
        for case in glob("tests/runit/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            // Skip the test runner
            if filename == "runner" {
                continue;
            }
            let name = format!("runit/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/runit").join(filename);
            let testcase = TestCase::new(name, case, output_file, TestType::Runtime, verbose);
            cases.push(testcase);
        }

        // Collect test-auxiliary
        let aux_use = regex::Regex::new(r"^//@\s*aux-build:(?P<fname>.*)").unwrap();
        let mut auxiliaries = vec![];
        for case in cases.iter() {
            let source = std::fs::read_to_string(&case.source).unwrap();
            for cap in aux_use.captures_iter(&source) {
                let fname = cap.name("fname").unwrap().as_str();
                let source = Path::new("tests/auxiliary").join(fname);
                let filename = source.file_stem().unwrap();
                let name = format!("auxiliary/{}", filename.to_string_lossy());

                // deduplication
                if auxiliaries.iter().any(|aux: &TestCase| aux.name == name) {
                    continue;
                }

                let output_file = manifest.out_dir.join(filename); // aux files are output to the base directory
                let testcase =
                    TestCase::new(name, source, output_file, TestType::CompileLib, verbose);
                auxiliaries.push(testcase);
            }
        }

        // Compile auxiliary before the tests
        let mut testcases = auxiliaries;
        testcases.extend(cases);
        testcases
    }

    fn bless(&self, update: bool, case: &TestCase) {
        let output = case.generated();
        let blessed = case.source.with_extension("c");

        self.log_action_context("checking", &blessed.display());
        if update {
            self.log_action_context("updating", &blessed.display());
            std::fs::copy(output, &blessed).unwrap();
            self.log_action_context("result", "updated");
        } else {
            let output = std::fs::read_to_string(output).unwrap();
            let blessed = std::fs::read_to_string(&blessed).unwrap();

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
            self.log_action_context("result", "passed");
        }
    }

    /// Run a runtime test and compare its output with the expected output
    fn run_and_check_output(&self, testcase: &TestCase) {
        // Run the test
        self.log_action_context("running", &testcase.output_file.display());
        let output = std::process::Command::new(&testcase.output_file)
            .output()
            .unwrap_or_else(|e| panic!("failed to run {}: {}", testcase.output_file.display(), e));

        // Check return value
        let actual_return = output.status.code().unwrap_or_else(|| {
            panic!("Process terminated by signal: {}", testcase.output_file.display())
        });

        let expected_return_path = testcase.source.with_extension("ret");
        if expected_return_path.exists() {
            self.log_action_context("checking return value", &expected_return_path.display());
            let expected_return = std::fs::read_to_string(&expected_return_path)
                .unwrap_or_else(|e| {
                    panic!("failed to read {}: {}", expected_return_path.display(), e)
                })
                .trim()
                .parse::<i32>()
                .unwrap_or_else(|e| {
                    panic!("invalid return value in {}: {}", expected_return_path.display(), e)
                });

            if actual_return != expected_return {
                cprintln!("<r,s>return value does not match expected value</r,s>");
                cprintln!("expected: {}", expected_return);
                cprintln!("actual: {}", actual_return);
                std::process::exit(1);
            }
            self.log_action_context("return value", "passed");
        }

        // Check stdout
        let actual_output = String::from_utf8_lossy(&output.stdout).into_owned();
        let expected_output_path = testcase.source.with_extension("out");

        if !expected_output_path.exists() {
            panic!("expected output file {} does not exist", expected_output_path.display());
        }

        self.log_action_context("checking stdout", &expected_output_path.display());
        let expected_output = std::fs::read_to_string(&expected_output_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", expected_output_path.display(), e));

        let diff = TextDiff::from_lines(&expected_output, &actual_output);
        if diff.ratio() < 1.0 {
            cprintln!("<r,s>stdout does not match expected output</r,s>");
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
        self.log_action_context("stdout", "passed");

        // Check stderr if there's any output
        if !output.stderr.is_empty() {
            let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
            let expected_stderr_path = testcase.source.with_extension("err");

            if expected_stderr_path.exists() {
                self.log_action_context("checking stderr", &expected_stderr_path.display());
                let expected_stderr = std::fs::read_to_string(&expected_stderr_path)
                    .unwrap_or_else(|e| {
                        panic!("failed to read {}: {}", expected_stderr_path.display(), e)
                    });

                let diff = TextDiff::from_lines(&expected_stderr, &stderr_str);
                if diff.ratio() < 1.0 {
                    cprintln!("<r,s>stderr does not match expected output</r,s>");
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
                self.log_action_context("stderr", "passed");
            } else if !stderr_str.trim().is_empty() {
                // If there's no .err file but we got stderr output, that's unexpected
                cprintln!("<r,s>unexpected stderr output:</r,s>");
                cprintln!("{}", stderr_str);
                std::process::exit(1);
            }
        }
        self.log_action_context("result", "all checks passed");
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
    /// Runtime test - compile, run and compare output
    Runtime,
}

impl TestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestType::Compile => "compile",
            TestType::CompileLib => "compile-lib",
            TestType::FileCheck => "filecheck",
            TestType::Bless => "bless",
            TestType::Runtime => "runtime",
        }
    }
}

pub struct TestCase {
    pub name: String,
    pub source: PathBuf,
    pub output_file: PathBuf,
    pub test: TestType,
    pub verbose: bool,
}

impl Run for TestCase {
    const STEP_DISPLAY_NAME: &'static str = "TESTCASE";
    fn run(&self, manifest: &Manifest) {
        self.build(manifest);
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}

impl TestCase {
    pub fn new(
        name: String,
        source: PathBuf,
        output_file: PathBuf,
        test: TestType,
        verbose: bool,
    ) -> Self {
        Self { name, source, output_file, test, verbose }
    }

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
        self.command_status("compile", &mut command);
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
        self.command_status("compile lib", &mut command);
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
    verbose: bool,
}

impl Run for FileChecker {
    const STEP_DISPLAY_NAME: &'static str = "FILECHECK";

    fn run(&self, _manifest: &Manifest) {}

    fn verbose(&self) -> bool {
        self.verbose
    }
}

impl FileChecker {
    pub fn new(verbose: bool) -> Self {
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

        Self { filecheck, verbose }
    }

    fn check_testcase(&self, case: &TestCase) {
        let generated = File::open(case.generated()).unwrap();
        let mut command = std::process::Command::new(&self.filecheck);
        command.arg(&case.source).stdin(generated);
        let output = self.command_output("filecheck", &mut command);
        assert!(
            output.status.success(),
            "failed to run FileCheck on {}",
            case.source.file_stem().unwrap().to_string_lossy()
        );
    }
}
