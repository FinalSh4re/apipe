//! A simple annonymous UNIX pipe type.
//!
//! ## Usage
//!
//! ```
//! use apipe::CommandPipe;
//!
//! fn main() {
//!
//!     let mut pipe = CommandPipe::new();
//!
//!     pipe.add_command("echo")
//!         .arg("This is a test.")
//!         .add_command("grep")
//!         .arg("-Eo")
//!         .arg(r"\w\w\sa[^.]*");
//!
//!     let output = pipe.spawn();
//!
//!     assert_eq!(
//!         String::from_utf8_lossy(&output.unwrap().stdout),
//!         String::from("is a test\n")
//!     );
//! }
//! ```

use anyhow::{anyhow, Context, Result};
use std::process::{Child, Command, Output, Stdio};

/// A type representing an annonymous pipe
pub struct CommandPipe {
    pipeline: Vec<Command>,
    length: usize,
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
            length: 0,
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
        self.length += 1;

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
    ///     .arg("My_Dir");
    ///
    /// let output = pipe.spawn();
    /// ```
    pub fn spawn(mut self) -> Result<Output> {
        let mut child: Option<Child> = None;
        let mut stdout = Stdio::null();

        for (i, command) in self.pipeline.iter_mut().enumerate() {
            child = Some(command.stdin(stdout).stdout(Stdio::piped()).spawn()?);

            child = match child {
                Some(mut c) => {
                    c.wait().with_context(|| {
                        format!(
                            "Child process '{}' exited with error code.",
                            command.get_program().to_string_lossy()
                        )
                    })?;
                    Some(c)
                }
                None => None,
            };

            stdout = if !(i == self.length - 1) {
                let stdin = child
                    .take()
                    .unwrap()
                    .stdout
                    .take()
                    .context("Couldn't read stdout of previous command.")?;

                Stdio::from(stdin)
            } else {
                Stdio::null()
            };
        }

        match child {
            Some(c) => c.wait_with_output().context("Failed to wait for process."),
            None => Err(anyhow!("No command in pipeline.")),
        }
    }

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
            .arg(r"\w\w\sa[^.]*");

        let output = pipe.spawn();

        assert_eq!(
            String::from_utf8_lossy(&output.unwrap().stdout),
            String::from("is a test\n")
        );
    }
}
