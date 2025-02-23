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
            }
            self.check_and_run_directives(&testcase);
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

        // Bless tests
        for case in glob("tests/bless/*.rs").unwrap() {
            let case = case.unwrap();
            let filename = case.file_stem().unwrap();
            let name = format!("bless/{}", filename.to_string_lossy());
            let output_file = manifest.out_dir.join("tests/bless").join(filename);
            let testcase = TestCase::new(name, case, output_file, TestType::Bless, verbose);
            cases.push(testcase);
        }

        // Collect and process auxiliary builds from directives
        let mut auxiliaries = vec![];
        for case in cases.iter() {
            let directives = case.parse_directives();
            for directive in directives {
                if let TestDirective::AuxBuild(fname) = directive {
                    let source = Path::new("tests/auxiliary").join(&fname);
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

    /// Run a runtime test and check its output against directives
    fn check_and_run_directives(&self, testcase: &TestCase) {
        // Parse directives from source
        let directives = testcase.parse_directives();
        self.log_action_context("directives", &format!("found {} directives", directives.len()));

        let mut runpass = false;
        let mut exitcode = None;
        let mut stdout = None;
        let mut stderr = None;

        // Check each directive
        for directive in directives {
            match directive {
                TestDirective::RunPass => runpass = true,
                TestDirective::CheckStdout(expected) => stdout = Some(expected),
                TestDirective::CheckStderr(expected) => stderr = Some(expected),
                TestDirective::ExitCode(expected) => exitcode = Some(expected),
                TestDirective::AuxBuild(_) => {
                    // AuxBuild directives are handled during test collection
                    // No need to check them during test execution
                }
            }
        }

        if !runpass && (exitcode.is_some() | stdout.is_some() | stderr.is_some()) {
            panic!("Directives conflicts, lack of '//@ run-pass'");
        }

        if runpass {
            self.run_and_check_output(testcase, exitcode, stdout, stderr);
        }

        self.log_action_context("result", "all checks passed");
    }

    fn run_and_check_output(
        &self,
        testcase: &TestCase,
        expected_exit: Option<i32>,
        expected_stdout: Option<String>,
        expected_stderr: Option<String>,
    ) {
        // Run the test
        self.log_action_context("running", &testcase.output_file.display());
        let output = std::process::Command::new(&testcase.output_file)
            .output()
            .unwrap_or_else(|e| panic!("failed to run {}: {}", testcase.output_file.display(), e));

        // Get actual outputs
        let actual_return = output.status.code().unwrap_or_else(|| {
            panic!("Process terminated by signal: {}", testcase.output_file.display())
        });
        let actual_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let actual_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        {
            let expected_exit = expected_exit.unwrap_or(0);
            self.log_action_context("checking exit code", &expected_exit.to_string());
            if actual_return != expected_exit {
                cprintln!("<r,s>exit code does not match expected value</r,s>");
                cprintln!("expected: {}", expected_exit);
                cprintln!("actual: {}", actual_return);
                std::process::exit(1);
            }
            self.log_action_context("exit code", "passed");
        }

        if let Some(expected_stdout) = expected_stdout {
            self.log_action_context("checking stdout", &expected_stdout);
            let diff = TextDiff::from_lines(&expected_stdout, &actual_stdout);
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
        }

        if let Some(expected_stderr) = expected_stderr {
            self.log_action_context("checking stderr", &expected_stderr);
            let diff = TextDiff::from_lines(&expected_stderr, &actual_stderr);
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
        }
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

impl TestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestType::Compile => "compile",
            TestType::CompileLib => "compile-lib",
            TestType::FileCheck => "filecheck",
            TestType::Bless => "bless",
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

    /// Parse test directives from the source file
    fn parse_directives(&self) -> Vec<TestDirective> {
        let source = std::fs::read_to_string(&self.source)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", self.source.display(), e));

        let mut directives = Vec::new();

        // Regular expressions for matching directives
        let run_pass = regex::Regex::new(r"^//@\s*run-pass").unwrap();
        let stdout_re = regex::Regex::new(r"^//@\s*check-stdout:\s*(.*)").unwrap();
        let stderr_re = regex::Regex::new(r"^//@\s*check-stderr:\s*(.*)").unwrap();
        let exit_re = regex::Regex::new(r"^//@\s*exit-code:\s*(\d+)").unwrap();
        let aux_re = regex::Regex::new(r"^//@\s*aux-build:\s*(.*)").unwrap();
        // Regex to match any directive pattern
        let directive_re = regex::Regex::new(r"^//@\s*([^:]+)").unwrap();

        for (line_num, line) in source.lines().enumerate() {
            if let Some(_cap) = run_pass.captures(line) {
                directives.push(TestDirective::RunPass);
            } else if let Some(cap) = stdout_re.captures(line) {
                let content = cap[1].trim().to_string();
                directives.push(TestDirective::CheckStdout(content));
            } else if let Some(cap) = stderr_re.captures(line) {
                let content = cap[1].trim().to_string();
                directives.push(TestDirective::CheckStderr(content));
            } else if let Some(cap) = exit_re.captures(line) {
                if let Ok(code) = cap[1].parse() {
                    directives.push(TestDirective::ExitCode(code));
                } else {
                    panic!(
                        "{}:{}: invalid exit code in directive",
                        self.source.display(),
                        line_num + 1
                    );
                }
            } else if let Some(cap) = aux_re.captures(line) {
                let fname = cap[1].trim().to_string();
                directives.push(TestDirective::AuxBuild(fname));
            } else if let Some(cap) = directive_re.captures(line) {
                let directive_name = cap[1].trim();
                panic!(
                    "{}:{}: unknown directive '{}', supported directives are: check-stdout, check-stderr, exit-code, aux-build",
                    self.source.display(),
                    line_num + 1,
                    directive_name
                );
            }
        }

        directives
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

/// Test directives that can appear in source files
#[derive(Debug)]
enum TestDirective {
    /// Compile and run a testcase,
    /// expect a success (exit with 0)
    RunPass,
    /// Expected stdout content
    CheckStdout(String),
    /// Expected stderr content
    CheckStderr(String),
    /// Expected exit code
    ExitCode(i32),
    /// Auxiliary build requirement
    AuxBuild(String),
}
