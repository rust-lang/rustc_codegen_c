use clap::Args;

use crate::{manifest::Manifest, Run};

/// Clean the build directory
#[derive(Args, Debug)]
pub struct CleanCommand {}

impl Run for CleanCommand {
    fn run(&self, manifest: &Manifest) {
        let _ = std::fs::remove_dir_all("crates/target");
        let _ = std::fs::remove_dir_all(&manifest.out_dir);
    }
}
