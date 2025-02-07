use anstream::eprintln as println;
use color_print::cprintln;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Manifest {
    pub verbose: bool,
    pub release: bool,
    pub out_dir: PathBuf,
}

impl Manifest {
    /// Builds the rustc codegen c library
    pub fn prepare(&self) {
        // Build codegen backend
        if self.verbose {
            cprintln!("<b>[BUILD]</b> preparing codegen backend");
            cprintln!("       target: {}", self.codegen_backend().display());
        } else {
            cprintln!("<b>[BUILD]</b> codegen backend");
        }

        let mut command = Command::new("cargo");
        command.arg("build").args(["--manifest-path", "crates/Cargo.toml"]);
        if self.verbose {
            command.args(["-v"]);
            cprintln!("       command: {}", format!("{:?}", command).replace('"', ""));
        }
        if self.release {
            command.arg("--release");
        }
        log::debug!("running {:?}", command);
        let status = command.status().unwrap();
        if self.verbose && status.success() {
            cprintln!("       <g>success</g>");
        }

        // Build runtime library
        if self.verbose {
            cprintln!("<b>[BUILD]</b> preparing librust_runtime");
            cprintln!("       output: {}", self.out_dir.display());
        } else {
            cprintln!("<b>[BUILD]</b> librust_runtime");
        }

        std::fs::create_dir_all(&self.out_dir).unwrap();
        let cc = std::env::var("CC").unwrap_or("clang".to_string());

        // Compile runtime.c
        let mut command = Command::new(&cc);
        command
            .arg("rust_runtime/rust_runtime.c")
            .arg("-o")
            .arg(self.out_dir.join("rust_runtime.o"))
            .arg("-c");
        if self.verbose {
            cprintln!("       compile: {}", format!("{:?}", command).replace('"', ""));
        }
        log::debug!("running {:?}", command);
        let status = command.status().unwrap();
        if self.verbose && status.success() {
            cprintln!("       <g>success</g>");
        }

        // Create static library
        let mut command = Command::new("ar");
        command
            .arg("rcs")
            .arg(self.out_dir.join("librust_runtime.a"))
            .arg(self.out_dir.join("rust_runtime.o"));
        if self.verbose {
            cprintln!("       archive: {}", format!("{:?}", command).replace('"', ""));
        }
        log::debug!("running {:?}", command);
        let status = command.status().unwrap();
        if self.verbose && status.success() {
            cprintln!("       <g>success</g>");
        }
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
