use clap::{Parser, Subcommand};

use crate::manifest::Manifest;

mod clean;
mod fmt;
mod manifest;
mod rustc;
mod test;

/// Bootstrap system for the rustc codegen c
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
pub struct Cli {
    /// Build the codegen backend in release mode
    #[arg(short, long)]
    pub release: bool,

    /// The output directory
    #[arg(short, long)]
    pub out_dir: Option<String>,

    /// verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Test(test::TestCommand),
    Clean(clean::CleanCommand),
    Rustc(rustc::RustcCommand),
    Fmt(fmt::FmtCommand),
}

trait Run {
    fn run(&self, manifest: &Manifest);
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let manifest = Manifest {
        verbose: cli.verbose,
        release: cli.release,
        out_dir: cli.out_dir.unwrap_or("build".to_string()).into(),
    };
    match cli.command {
        Command::Test(test) => test.run(&manifest),
        Command::Clean(clean) => clean.run(&manifest),
        Command::Rustc(rustc) => rustc.run(&manifest),
        Command::Fmt(fmt) => fmt.run(&manifest),
    }
}
