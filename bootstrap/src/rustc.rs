use std::path::PathBuf;

use clap::Args;
use color_print::cprintln;

use crate::log::Log;
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

impl Log for RustcCommand {
    fn log_step(&self, step_type: &str, name: &str, details: Vec<(&str, &str)>) {
        if self.verbose {
            cprintln!("<b>[RUSTC]</b> {} {}", step_type, name);
            for (label, value) in details {
                cprintln!("       {}: {}", label, value);
            }
        }
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

impl Run for RustcCommand {
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

        let status = command.status().unwrap();
        self.log_command("rustc", &command, &Some(status));
    }
}
