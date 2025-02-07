use color_print::cprintln;
use std::process::{Command, ExitStatus};

/// output log trait
pub trait Log {
    /// log command
    fn log_command(&self, prefix: &str, command: &Command, status: &Option<ExitStatus>) {
        if self.is_verbose() {
            cprintln!("       {}: {}", prefix, format!("{:?}", command).replace('"', ""));
        }
        if let Some(status) = status {
            if status.success() {
                cprintln!("       <g>success</g>");
            } else {
                cprintln!("       <r>failed</r>");
            }
        }
    }

    /// log step(step_type: step type, name: step name, details: step details)
    fn log_step(&self, step_type: &str, name: &str, details: Vec<(&str, &str)>);

    /// is verbose
    fn is_verbose(&self) -> bool;
}
