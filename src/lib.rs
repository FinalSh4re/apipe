//! A simple annonymous UNIX pipe type.
//!
//! ## Usage
//!
//! ```
//! let mut pipe = apipe::CommandPipe::new();
//!
//! pipe.add_command("echo")
//!     .arg("This is a test.")
//!     .add_command("grep")
//!     .arg("-Eo")
//!     .arg(r"\w\w\sa[^.]*")
//!     .spawn()
//!     .expect("Failed to spawn pipe.");
//!     
//! let output = pipe.output();
//!     
//! assert_eq!(
//!     output.unwrap(),
//!     "is a test\n"
//! );
//! ```

use anyhow::{anyhow, Context, Result};
use std::{
    io::Read,
    process::{Child, Command, Stdio},
};
#[derive(Debug, Default)]
/// A type representing an annonymous pipe
pub struct CommandPipe {
    pipeline: Vec<Command>,
    spawned_processes: Vec<Child>,
    output: Option<String>,
}

impl CommandPipe {
    /// Create a new empty pipe.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use apipe::CommandPipe;
    /// let mut pipe = CommandPipe::new();
    /// ```
    pub fn new() -> Self {
        CommandPipe {
            pipeline: Vec::new(),
            spawned_processes: Vec::new(),
            output: None,
        }
    }

    /// Add a command to the pipe.
    ///
    /// The command is passed eiter as an absolute path or as a relative path.
    /// For relative paths the PATH is checked.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// let mut pipe = CommandPipe::new();
    /// pipe.add_command("ls");
    /// ```
    pub fn add_command(&mut self, c: &str) -> &mut Self {
        let command = Command::new(c);
        self.pipeline.push(command);

        self
    }

    /// Add a single arguement to the preceding command in the pipe.
    ///
    /// Arguments need to be passed one at a time.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// let mut pipe = CommandPipe::new();
    /// pipe.add_command("ls").arg("-la");
    /// ```
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.pipeline
            .last_mut()
            .expect("No Command in pipe to add args to.")
            .arg(arg);
        self
    }

    /// Add multiple arguements to the preceding command in the pipe.
    ///
    /// Arguments are passed as a sequence.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// let mut pipe = CommandPipe::new();
    /// pipe.add_command("ls").args(vec!["-la", "~/Documents"]);
    /// ```
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.pipeline
            .last_mut()
            .expect("No Command in pipe to add args to.")
            .args(args);
        self
    }

    /// Runs the commands in the pipe and returns the output.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// let mut pipe = CommandPipe::new();
    /// pipe.add_command("ls")
    ///     .args(vec!["-la", "~/Documents"])
    ///     .add_command("grep")
    ///     .arg("My_Dir")
    ///     .spawn()
    ///     .expect("Failed to spawn pipe.");
    /// ```
    pub fn spawn(&mut self) -> Result<()> {
        for command in self.pipeline.iter_mut() {
            let stdin = match self.spawned_processes.last_mut() {
                Some(proc) => {
                    let stdout = proc
                        .stdout
                        .take()
                        .context("Failed to get stdout of previous process.")?;

                    Stdio::from(stdout)
                }
                None => Stdio::null(),
            };

            let mut child = command.stdin(stdin).stdout(Stdio::piped()).spawn()?;

            child.wait().with_context(|| {
                format!(
                    "Child process '{}' exited with error code.",
                    command.get_program().to_string_lossy()
                )
            })?;

            self.spawned_processes.push(child);
        }

        Ok(())
    }

    pub fn output(&mut self) -> Result<&str> {
        match &self.output {
            None => {
                if let Some(proc) = self.spawned_processes.last_mut() {
                    let mut output = String::new();
                    proc.stdout
                        .as_mut()
                        .context("Process isn't running")?
                        .read_to_string(&mut output)
                        .context("Failed to read stdout of final command.")?;
                    self.output.replace(output);

                    Ok(self.output.as_ref().unwrap())
                } else {
                    Err(anyhow!("No spawned processes!"))
                }
            }

            Some(_) => Ok(self.output.as_ref().unwrap()),
        }
    }

    #[cfg(test)]
    // Returns a [Vec] with references to all the commands currently in the pipeline.
    pub fn get_pipeline(&self) -> Vec<&Command> {
        self.pipeline.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").arg("-la").arg("~/Documents");

        let args: Vec<&std::ffi::OsStr> = pipe.get_pipeline()[0].get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_args() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").args(vec!["-la", "~/Documents"]);

        let args: Vec<&std::ffi::OsStr> = pipe.get_pipeline()[0].get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_pipe() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("echo")
            .arg("This is a test.")
            .add_command("grep")
            .arg("-Eo")
            .arg(r"\w\w\sa[^.]*")
            .spawn()
            .expect("Failed to spawn pipe.");

        let output = pipe.output();

        assert_eq!(output.unwrap(), "is a test\n");
    }
}
