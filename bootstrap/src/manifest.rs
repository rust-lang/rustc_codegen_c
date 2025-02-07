use anstream::eprintln as println;
use color_print::cprintln;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::log::Log;

#[derive(Debug)]
pub struct Manifest {
    pub verbose: bool,
    pub release: bool,
    pub out_dir: PathBuf,
}

impl Log for Manifest {
    fn log_step(&self, step_type: &str, name: &str, details: Vec<(&str, &str)>) {
        if self.verbose {
            cprintln!("<b>[BUILD]</b> {} {}", step_type, name);
            for (label, value) in details {
                cprintln!("       {}: {}", label, value);
            }
        } else {
            cprintln!("<b>[BUILD]</b> {} {}", step_type, name);
        }
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

impl Manifest {
    /// Builds the rustc codegen c library
    pub fn prepare(&self) {
        // New step
        // Build codegen backend
        self.log_step(
            "preparing",
            "codegen backend",
            vec![("target", &self.codegen_backend().display().to_string())],
        );

        let mut command = Command::new("cargo");
        command.arg("build").args(["--manifest-path", "crates/Cargo.toml"]);
        if self.verbose {
            command.args(["-v"]);
        }
        if self.release {
            command.arg("--release");
        }
        let status = command.status().unwrap();
        let status = if self.verbose { Some(status) } else { None };
        self.log_command("command", &command, &status);

        // New step
        // Build runtime library
        self.log_step(
            "librust_runtime",
            "librust_runtime",
            vec![("output", &self.out_dir.display().to_string())],
        );

        // cmd: Create output directory
        match std::fs::create_dir_all(&self.out_dir) {
            Ok(_) => (),
            Err(e) => {
                cprintln!("       <r>failed</r> to create output directory: {}", e);
                std::process::exit(1);
            }
        }
        let cc = std::env::var("CC").unwrap_or("clang".to_string());

        // cmd: Compile runtime.c
        let mut command = Command::new(&cc);
        command
            .arg("rust_runtime/rust_runtime.c")
            .arg("-o")
            .arg(self.out_dir.join("rust_runtime.o"))
            .arg("-c");
        let status = command.status().unwrap();
        let status = if self.verbose { Some(status) } else { None };
        self.log_command("compile", &command, &status);

        // cmd: Create static library
        let mut command = Command::new("ar");
        command
            .arg("rcs")
            .arg(self.out_dir.join("librust_runtime.a"))
            .arg(self.out_dir.join("rust_runtime.o"));
        let status = command.status().unwrap();
        let status = if self.verbose { Some(status) } else { None };
        self.log_command("archive", &command, &status);
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
