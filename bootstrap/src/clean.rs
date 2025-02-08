use clap::Args;

use crate::manifest::Manifest;
use crate::Run;

/// Clean the build directory
#[derive(Args, Debug)]
pub struct CleanCommand {
    #[arg(short, long)]
    pub verbose: bool,
}

impl Run for CleanCommand {
    const STEP_DISPLAY_NAME: &'static str = "clean";

    fn run(&self, manifest: &Manifest) {
        self.log_action_start("cleaning", "build directory");
        let _ = std::fs::remove_dir_all("crates/target");
        self.log_action_context("rm", "crates/target");
        let _ = std::fs::remove_dir_all(&manifest.out_dir);
        self.log_action_context("rm", &manifest.out_dir.display());
    }

    fn verbose(&self) -> bool {
        self.verbose
    }
}
