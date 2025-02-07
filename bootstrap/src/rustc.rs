use std::path::PathBuf;

use clap::Args;

use crate::manifest::Manifest;
use crate::Run;

/// Invoke rustc
#[derive(Args, Debug)]
pub struct RustcCommand {
    source: PathBuf,

    #[arg(last = true)]
    slop: Vec<String>,

    #[arg(short, long)]
    pub verbose: bool,
}

impl Run for RustcCommand {
    const STEP_DISPLAY_NAME: &'static str = "RUSTC";

    fn run(&self, manifest: &Manifest) {
        manifest.prepare();

        let mut command = manifest.rustc();
        command
            .arg(&self.source)
            .args(["--crate-type", "bin"])
            .arg("--out-dir")
            .arg(&manifest.out_dir)
            .args(&self.slop);
        if self.verbose {
            command.env("RUST_BACKTRACE", "full");
        }

        self.command_status("rustc", &mut command);
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}
