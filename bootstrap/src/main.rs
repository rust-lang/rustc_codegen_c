use std::fmt::Display;
use std::process::{self, ExitStatus};

use clap::{Parser, Subcommand};
use color_print::cprintln;

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
    // The name like "BUILD" or "TEST" for logs.
    const STEP_DISPLAY_NAME: &'static str;
    fn run(&self, manifest: &Manifest);

    /// True if verbose output should be enabled.
    fn verbose(&self) -> bool;

    /// Record that the step has started a new action.
    fn log_action_start(&self, action: &str, item: impl Display) {
        let name = Self::STEP_DISPLAY_NAME;
        cprintln!("<b>[{name}]</b> {action} <cyan>{item}</cyan>");
    }

    /// Record context associated with the current action. Only use if there has been a preceding
    /// call to `log_action_start`.
    fn log_action_context(&self, key: impl Display, value: impl Display) {
        if self.verbose() {
            cprintln!("       {key}: {value}");
        }
    }

    /// Run a command and ensure it succeeds, capturing output.
    fn command_output(&self, action: &str, command: &mut process::Command) -> process::Output {
        if self.verbose() {
            cprintln!("       {action}: {command:?}");
        }

        match command.output() {
            // Command ran and completed successfully
            Ok(output) if output.status.success() => {
                if self.verbose() {
                    cprintln!("       <g>success</g>");
                }
                output
            }
            // Command ran but did not complete
            Ok(output) => panic!("command failed: {output:?}"),
            Err(e) => panic!("command failed: {e:?}"),
        }
    }

    /// Run a command and ensure it succeeds.
    fn command_status(&self, action: &str, command: &mut process::Command) -> ExitStatus {
        if self.verbose() {
            cprintln!("       {}: {}", action, format!("{:?}", command).replace('"', ""));
        }
        match command.status() {
            // Command ran and completed successfully
            Ok(status) if status.success() => {
                if self.verbose() {
                    cprintln!("       <g>success</g>");
                }
                status
            }
            // Command ran but did not complete
            Ok(status) => panic!("command failed: {status:?}"),
            Err(e) => panic!("command failed: {e:?}"),
        }
    }
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
        Command::Test(mut test) => {
            test.verbose |= cli.verbose;
            test.run(&manifest)
        }
        Command::Clean(mut clean) => {
            clean.verbose |= cli.verbose;
            clean.run(&manifest)
        }
        Command::Rustc(mut rustc) => {
            rustc.verbose |= cli.verbose;
            rustc.run(&manifest)
        }
        Command::Fmt(mut fmt) => {
            fmt.verbose |= cli.verbose;
            fmt.run(&manifest)
        }
    }
}
