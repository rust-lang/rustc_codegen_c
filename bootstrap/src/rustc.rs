use std::path::PathBuf;

use clap::Args;

use crate::manifest::Manifest;

/// Invoke rustc
#[derive(Args, Debug)]
pub struct RustcCommand {
    source: PathBuf,

    #[arg(last = true)]
    slop: Vec<String>,
}

impl RustcCommand {
    pub fn run(&self, manifest: &Manifest) {
        manifest.prepare();

        let mut command = manifest.rustc();
        command
            .arg(&self.source)
            .args(["--crate-type", "bin"])
            .arg("--out-dir")
            .arg(&manifest.out_dir)
            .args(&self.slop);
        log::debug!("running {:?}", command);
        command.status().unwrap();
    }
}
