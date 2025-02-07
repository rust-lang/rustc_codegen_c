use std::path::{Path, PathBuf};
use std::process::Command;

use anstream::eprintln as println;
use color_print::cprintln;

use crate::Run;

#[derive(Debug)]
pub struct Manifest {
    pub verbose: bool,
    pub release: bool,
    pub out_dir: PathBuf,
}

impl Manifest {
    /// Builds the rustc codegen c library
    pub fn prepare(&self) {
        let prepare = PrepareAction { verbose: self.verbose };
        prepare.run(&self);
    }

    /// The path to the rustc codegen c library
    pub fn codegen_backend(&self) -> &'static Path {
        if self.release {
            Path::new("crates/target/release/librustc_codegen_c.so")
        } else {
            Path::new("crates/target/debug/librustc_codegen_c.so")
        }
    }

    /// The command to run rustc with the codegen backend
    pub fn rustc(&self) -> Command {
        let mut command = Command::new("rustc");
        command
            .args(["--edition", "2021"])
            .arg("-Z")
            .arg(format!("codegen-backend={}", self.codegen_backend().display()))
            .args(["-C", "panic=abort"])
            .args(["-C", "lto=false"])
            .arg(format!("-Lall={}", self.out_dir.display()))
            .env("CFLAGS", "-Irust_runtime")
            .arg("-lc")
            .arg("-lrust_runtime");
        if self.verbose {
            command.env("RUST_BACKTRACE", "full");
        }
        command
    }
}

struct PrepareAction {
    verbose: bool,
}

impl Run for PrepareAction {
    const STEP_DISPLAY_NAME: &'static str = "prepare";

    fn run(&self, manifest: &Manifest) {
        // action: Build codegen backend
        self.log_action_start("building", "codegen backend");
        self.log_action_context("target", manifest.codegen_backend().display());

        let mut command = Command::new("cargo");
        command.arg("build").args(["--manifest-path", "crates/Cargo.toml"]);
        if manifest.verbose {
            command.args(["-v"]);
        }
        if manifest.release {
            command.arg("--release");
        }
        self.command_status("build", &mut command);

        // action: Build runtime library
        self.log_action_start("building", "librust_runtime");
        self.log_action_context("output dir", &manifest.out_dir.to_path_buf().display());

        // cmd: Create output directory
        if let Err(e) = std::fs::create_dir_all(&manifest.out_dir) {
            cprintln!("       <r>failed</r> to create output directory: {}", e);
            std::process::exit(1);
        }

        let cc = std::env::var("CC").unwrap_or("clang".to_string());

        // cmd: Compile runtime.c
        let mut command = Command::new(&cc);
        command
            .arg("rust_runtime/rust_runtime.c")
            .arg("-o")
            .arg(manifest.out_dir.join("rust_runtime.o"))
            .arg("-c");
        self.command_status("build", &mut command);

        // cmd: Create static library
        let mut command = Command::new("ar");
        command
            .arg("rcs")
            .arg(manifest.out_dir.join("librust_runtime.a"))
            .arg(manifest.out_dir.join("rust_runtime.o"));
        self.command_status("archive", &mut command);
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}
