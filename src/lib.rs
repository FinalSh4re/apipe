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
//! let output = pipe.output().unwrap().stdout();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
//! ```

use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{anyhow, Context, Result};
use std::{process::{Child, Stdio}, ffi::OsStr};



#[derive(Debug, Default)]
/// A type representing an annonymous pipe
pub struct CommandPipe {
    pipeline: Vec<Command>,
    last_spawned: Option<Child>,
    output: Option<Output>,
}

#[derive(Debug)]
pub struct Command(std::process::Command);

impl From<std::process::Command> for Command {
    fn from(command: std::process::Command) -> Self {
        Command(command)
    }
}


impl Command {

    pub fn new<S>(command: S) -> Self 
    where
        S: AsRef<OsStr>
    {
        Command(std::process::Command::new(command))
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Self 
    where
        S: AsRef<OsStr>
    {
        self.0.arg(arg);
        self
    }
    
    pub fn args<S, I>(&mut self, args: I) -> &mut Self 
    where 
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        self.0.args(args);
        self

    }

    pub fn from_str(c: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"([^\s"']+)|("[^"]*?")|('[^']*?')"#).unwrap();
        }
        
        let matches = RE.captures_iter(c);
        let cmd_parts: Vec<&str> = matches.map(|x| x.get(0).unwrap().as_str()).collect();

        let (cmd, args) = cmd_parts.split_first().context("Invalid command string: No command found.")?;

        let mut command = Command::new(cmd);
        
        command.args(args);
        
        Ok(command)

    }
}

#[derive(Debug)]
pub struct Output(std::process::Output);

impl From<std::process::Output> for Output {
    fn from(command: std::process::Output) -> Self {
        Output(command)
    }
}

impl Output {
    pub fn status_code(&self) -> Option<i32> {
        self.0.status.code()
    }

    pub fn stdout(&self) -> &[u8] {
        self.0.stdout.as_slice()
    }

    pub fn stderr(&self) -> &[u8] {
        self.0.stderr.as_slice()
    }
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
    pub fn add_command<S>(&mut self, c: S) -> &mut Self 
    where
        S: AsRef<std::ffi::OsStr>
    {
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
    pub fn arg<S>(&mut self, arg: S) -> &mut Self 
    where
        S: AsRef<std::ffi::OsStr>
    {
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

    /// Runs the commands in the pipe.
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

            let mut child = command.0.stdin(stdin).stdout(Stdio::piped()).spawn()?;

            child.wait().with_context(|| {
                format!(
                    "Child process '{}' exited with error code.",
                    command.0.get_program().to_string_lossy()
                )
            })?;

            self.last_spawned.replace(child);
        }

        Ok(())
    }

    /// Returns the output of the pipe.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// let mut pipe = apipe::CommandPipe::new();
    ///
    /// pipe.add_command("echo")
    ///     .arg("This is a test.")
    ///     .add_command("grep")
    ///     .arg("-Eo")
    ///     .arg(r"\w\w\sa[^.]*")
    ///     .spawn()
    ///     .expect("Failed to spawn pipe.");
    ///     
    /// let output = pipe.output().unwrap().stdout();
    ///     
    /// assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
    /// ```
    pub fn output(&mut self) -> Result<&Output> {
        match self.output {
            Some(_) => Ok(self.output.as_ref().unwrap()),
            None => {
                if let Some(last_proc) = self.last_spawned.take() {
                    let output = last_proc.wait_with_output()?;

                    self.output.replace(Output::from(output));
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

        let args: Vec<&std::ffi::OsStr> = pipe.pipeline[0].0.get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_args() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").args(vec!["-la", "~/Documents"]);

        let args: Vec<&std::ffi::OsStr> = pipe.pipeline[0].0.get_args().collect();

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

        let output = pipe.output().unwrap().stdout();

        assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
    }
}
