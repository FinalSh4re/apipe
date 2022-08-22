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
//! let output = pipe.output().unwrap().stdout.as_slice();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
//! ```

use anyhow::{anyhow, Context, Result};
use std::process::{Child, Command, Output, Stdio};

#[derive(Debug, Default)]
/// A type representing an annonymous pipe
pub struct CommandPipe {
    pipeline: Vec<Command>,
    last_spawned: Option<Child>,
    output: Option<Output>,
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
            last_spawned: None,
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

    /// Add a single argument to the preceding command in the pipe.
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

    /// Add multiple arguments to the preceding command in the pipe.
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
            let stdin = match self.last_spawned.take() {
                Some(mut proc) => {
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

            self.last_spawned.replace(child);
        }

        Ok(())
    }

    pub fn output(&mut self) -> Result<&Output> {
        match self.output {
            Some(_) => Ok(self.output.as_ref().unwrap()),
            None => {
                if let Some(last_proc) = self.last_spawned.take() {
                    let output = last_proc.wait_with_output()?;

                    self.output.replace(output);
                    self.output()
                } else {
                    Err(anyhow!("No spawned process in pipeline"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").arg("-la").arg("~/Documents");

        let args: Vec<&std::ffi::OsStr> = pipe.pipeline[0].get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_args() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").args(vec!["-la", "~/Documents"]);

        let args: Vec<&std::ffi::OsStr> = pipe.pipeline[0].get_args().collect();

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

        let output = pipe.output().unwrap().stdout.as_slice();

        assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
    }
}
