use std::process::Command;

use clap::Args;
use glob::glob;

use crate::Run;

/// Format code, examples and tests
#[derive(Args, Debug)]
pub struct FmtCommand {
    #[arg(short, long)]
    pub check: bool,
}

impl Run for FmtCommand {
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
}

impl FmtCommand {
    pub fn perform(&self, command: &mut Command) {
        if self.check {
            command.arg("--check");
        }
        log::debug!("running {:?}", command);
        assert!(command.status().unwrap().success(), "failed to run {:?}", command);
    }
}
