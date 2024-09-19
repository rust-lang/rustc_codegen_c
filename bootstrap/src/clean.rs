use clap::Args;

use crate::manifest::Manifest;

/// Clean the build directory
#[derive(Args, Debug)]
pub struct CleanCommand {}

impl CleanCommand {
    pub fn run(&self, manifest: &Manifest) {
        std::fs::remove_dir_all("crates/target").unwrap();
        std::fs::remove_dir_all(&manifest.out_dir).unwrap();
    }
}
