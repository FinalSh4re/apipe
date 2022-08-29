//! An anonymous pipe.

use crate::{cmd::Command, error::APipeError, output::Output};
use std::{
    ffi::OsStr,
    ops,
    process::{Child, Stdio},
};

type Result<T> = std::result::Result<T, APipeError>;

#[derive(Debug, Default)]
/// A type representing an anonymous pipe
pub struct CommandPipe {
    pub(crate) pipeline: Vec<Command>,
    last_spawned: Option<Child>,
}

impl ops::BitOr<Command> for CommandPipe {
    type Output = CommandPipe;

    fn bitor(mut self, rhs: Command) -> Self {
        self.pipeline.push(rhs);
        self
    }
}

#[cfg(feature = "parser")]
impl TryFrom<&str> for CommandPipe {
    type Error = APipeError;

    fn try_from(value: &str) -> Result<Self> {
        let mut pipe = CommandPipe::new();

        for cmd in value.split_terminator("|") {
            match Command::parse_str(cmd) {
                Ok(c) => pipe.pipeline.push(c),
                Err(e) => return Err(e),
            }
        }
        Ok(pipe)
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
        S: AsRef<OsStr>,
    {
        let command = c.into();
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
        S: AsRef<OsStr>,
    {
        let command = self
            .pipeline
            .pop()
            .expect("No Command in pipe to add args to.");

        self.pipeline.push(command.arg(arg));
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
        S: AsRef<OsStr>,
    {
        let command = self
            .pipeline
            .pop()
            .expect("No Command in pipe to add args to.");

        self.pipeline.push(command.args(args));
        self
    }

    /// Runs the commands in the pipe.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// # fn main() -> Result<(), apipe::error::APipeError> {
    /// let mut pipe = CommandPipe::new();
    /// pipe.add_command("ls")
    ///     .args(vec!["-la", "~/Documents"])
    ///     .add_command("grep")
    ///     .arg("My_Dir")
    ///     .spawn()
    ///     .expect("Failed to spawn pipe.");
    /// # Ok(())
    /// # }
    /// ```
    pub fn spawn(&mut self) -> Result<()> {
        for command in self.pipeline.iter_mut() {
            let stdin = self.last_spawned.take().map_or(Stdio::null(), |mut std| {
                std.stdout.take().map_or(Stdio::null(), Stdio::from)
            });

            let mut child = command
                .0
                .stdin(stdin)
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|e| APipeError::ChildProcess(e, "Failed to spawn child command"))?;

            child.wait().map_err(|e| {
                APipeError::ChildProcess(e, "Child process exited with error code.")
            })?;

            self.last_spawned.replace(child);
        }

        Ok(())
    }

    /// Spawns all commands in the pipe and returns the [`Output`].
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::CommandPipe;
    /// # fn main() -> Result<(), apipe::error::APipeError> {
    /// let output = CommandPipe::new()
    ///     .add_command("echo")
    ///     .arg("This is a test.")
    ///     .add_command("grep")
    ///     .arg("-Eo")
    ///     .arg(r"\w\w\sa[^.]*")
    ///     .spawn_with_output()?;
    ///
    /// assert_eq!(output.stdout(), "is a test\n".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    pub fn spawn_with_output(&mut self) -> Result<Output> {
        self.spawn()?;
        self.output()
    }

    /// Returns the [`Output`] of the pipe.
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
    /// let output = pipe.output().unwrap();
    ///     
    /// assert_eq!(output.stdout(), "is a test\n".as_bytes());
    /// ```
    pub fn output(&mut self) -> Result<Output> {
        if let Some(last_proc) = self.last_spawned.take() {
            let output = last_proc.wait_with_output().map_err(|e| {
                APipeError::ChildProcess(e, "Child process exited with error code.")
            })?;
            return Ok(Output::from(output));
        } else {
            Err(APipeError::NoRunningProcesses)
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

        let args: Vec<&OsStr> = pipe.pipeline[0].0.get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_args() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").args(vec!["-la", "~/Documents"]);

        let args: Vec<&OsStr> = pipe.pipeline[0].0.get_args().collect();

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

        let output = pipe.output().unwrap();

        assert_eq!(output.stdout(), "is a test\n".as_bytes());
    }

    #[test]
    #[should_panic]
    fn test_add_arg_without_command() {
        let mut pipe = CommandPipe::new();
        pipe.arg("ls");
    }

    #[test]
    fn test_spawn_with_output() {
        let output = CommandPipe::new()
            .add_command("echo")
            .arg("This is a test.")
            .add_command("grep")
            .arg("-Eo")
            .arg(r"\w\w\sa[^.]*")
            .spawn_with_output()
            .unwrap();

        assert_eq!(output.stdout(), "is a test\n".as_bytes());
    }

    #[test]
    fn test_overload() {
        let mut pipe = CommandPipe::new();

        pipe = pipe | Command::new("grep");

        assert_eq!(pipe.pipeline[0].0.get_program(), "grep");

        let output = (Command::new("echo").arg("This is a test.")
            | Command::new("grep").args(&["-Eo", r"\w\w\sa[^.]*"]))
        .spawn_with_output()
        .unwrap();

        assert_eq!(output.stdout(), "is a test\n".as_bytes());
    }

    #[cfg(feature = "parser")]
    #[test]
    fn test_try_from() {
        let mut pipe =
            CommandPipe::try_from(r#"echo "This is a test." | grep -Eo \w\w\sa[^.]*"#).unwrap();
        let output = pipe.spawn_with_output().unwrap();

        assert_eq!(output.stdout(), "is a test\n".as_bytes());
    }

    #[cfg(feature = "parser")]
    #[test]
    fn test_try_from_command() {
        let mut pipe = CommandPipe::try_from(r#"echo "This is a test."#).unwrap();
        let output = pipe.spawn_with_output().unwrap();

        assert_eq!(output.stdout(), "This is a test.\n".as_bytes());
    }

    #[cfg(feature = "parser")]
    #[test]
    fn test_try_from_empty_str() {
        let pipe = CommandPipe::try_from("");

        if let Err(_) = pipe {
            panic!("Pipe should be empty!")
        };
    }

    #[cfg(feature = "parser")]
    #[test]
    fn test_try_from_invalid_pipe() {
        let pipe = CommandPipe::try_from(" | ");

        if let Ok(_) = pipe {
            panic!("Shouldn't be able to parse invalid pipe!")
        };
    }
}
