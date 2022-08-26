use crate::{error::APipeError, pipe::CommandPipe};
use lazy_static::lazy_static;
use regex::Regex;
use std::{ffi::OsStr, ops};

type Result<T> = std::result::Result<T, APipeError>;

/// Abstraction of an external command.
///
/// ## Example
///
/// ```
/// # use apipe::{Command, error::APipeError};
/// # fn main() -> Result<(), APipeError> {
/// let cmd = Command::parse_str(r#"grep -Eo \w\w\sa[^.]*"#)?;
///
/// // or
///
/// let cmd = Command::new("grep").args(&["-Eo", r"\w\w\sa[^.]*"]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Command(pub(crate) std::process::Command);

impl<T> From<T> for Command
where
    T: AsRef<OsStr>,
{
    fn from(s: T) -> Self {
        Command(std::process::Command::new(s))
    }
}

impl ops::BitOr<Command> for Command {
    type Output = CommandPipe;

    fn bitor(self, rhs: Self) -> CommandPipe {
        let mut pipe = CommandPipe::new();
        pipe.pipeline.push(self);
        pipe.pipeline.push(rhs);

        pipe
    }
}

impl Command {
    /// Creates a new command instance.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::Command;
    /// let cmd = Command::new("ls");
    /// ```
    pub fn new<S>(command: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        Command(std::process::Command::new(command))
    }

    /// Adds a single argument to an existing Command instance.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::Command;
    /// let cmd = Command::new("ls").arg("-la");
    /// ```
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr>,
    {
        self.0.arg(arg);
        self
    }

    /// Adds a multiple arguments to an existing Command instance.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::Command;
    /// let cmd = Command::new("grep").args(&["-Eo", r"\w\w\sa[^.]*"]);
    /// ```
    pub fn args<S, I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.0.args(args);
        self
    }

    /// Constructs a Command from a string including the program and its args.
    ///
    /// ## Example
    ///
    /// ```
    /// # use apipe::{cmd::Command, error::APipeError};
    /// # fn main() -> Result<(), APipeError> {
    ///     let cmd = Command::parse_str(r#"echo "This is a test.""#)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_str(c: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"([^\s"']+)|("[^"]*?")|('[^']*?')"#).unwrap();
        }

        let matches = RE.captures_iter(c);
        let cmd_parts: Vec<&str> = matches.map(|x| x.get(0).unwrap().as_str()).collect();

        let (&cmd, args) = cmd_parts
            .split_first()
            .ok_or_else(|| APipeError::Parser(c.to_owned()))?;

        let mut command = Command::new(cmd);

        command.args(args);

        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_pipe() -> Result<()> {
        let pipe = Command::parse_str(r#"echo "This is a test.""#)?
            | Command::parse_str(r#"grep -Eo \w\w\sa[^.]*"#)?;

        assert_eq!(pipe.pipeline[0].0.get_program(), "echo");
        assert_eq!(
            pipe.pipeline[0].0.get_args().collect::<Vec<&OsStr>>(),
            &["\"This is a test.\""]
        );
        assert_eq!(pipe.pipeline[1].0.get_program(), "grep");
        assert_eq!(
            pipe.pipeline[1].0.get_args().collect::<Vec<&OsStr>>(),
            &["-Eo", r"\w\w\sa[^.]*"]
        );

        Ok(())
    }
}
