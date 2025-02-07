use std::process::Command;

use clap::Args;
use glob::glob;

use crate::Run;

/// Format code, examples and tests
#[derive(Args, Debug)]
pub struct FmtCommand {
    #[arg(short, long)]
    pub check: bool,

    #[arg(short, long)]
    pub verbose: bool,
}

impl Run for FmtCommand {
    const STEP_DISPLAY_NAME: &'static str = "FMT";

    fn run(&self, _manifest: &crate::manifest::Manifest) {
        self.perform(
            Command::new("cargo").arg("fmt").args(["--manifest-path", "bootstrap/Cargo.toml"]),
        );
        self.perform(
            Command::new("cargo")
                .arg("fmt")
                .args(["--manifest-path", "crates/Cargo.toml"])
                .arg("--all"),
        );
        for file in glob("examples/**/*.rs").unwrap() {
            self.perform(Command::new("rustfmt").args(["--edition", "2021"]).arg(file.unwrap()));
        }
        for file in glob("tests/**/*.rs").unwrap() {
            self.perform(Command::new("rustfmt").args(["--edition", "2021"]).arg(file.unwrap()));
        }
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}

impl FmtCommand {
    pub fn perform(&self, command: &mut Command) {
        if self.check {
            command.arg("--check");
        }

        self.command_status("format code", command);
    }
}
