use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};
use std::ffi::OsStr;
use crate::pipe::CommandPipe;
use std::ops;


/// Abstraction of an external command.
/// 
/// ## Example
/// 
/// ```
/// # use apipe::Command;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Either use by "piping" one Command type to another
/// let pipe = Command::from_str(r#"echo "This is a test.""#)? | Command::from_str(r#"grep -Eo \w\w\sa[^.]*"#)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Command(pub(crate) std::process::Command);

impl From<std::process::Command> for Command {
    fn from(command: std::process::Command) -> Self {
        Command(command)
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


    /// Constructs a Command from a string including the program and its args.
    /// 
    /// ## Example
    /// 
    /// ```
    /// # use apipe::cmd::Command;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    ///     let cmd = Command::from_str(r#"echo "This is a test.""#)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_str(c: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"([^\s"']+)|("[^"]*?")|('[^']*?')"#).unwrap();
        }
    
        let matches = RE.captures_iter(c);
        let cmd_parts: Vec<&str> = matches.map(|x| x.get(0).unwrap().as_str()).collect();

        let (&cmd, args) = cmd_parts.split_first().context("Invalid command string: No command found.")?;

        let mut command = Command::new(cmd);
    
        command.args(args);
    
        Ok(command)

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_pipe() -> Result<(), Box<dyn std::error::Error>> {
        let pipe = Command::from_str(r#"echo "This is a test.""#)? | Command::from_str(r#"grep -Eo \w\w\sa[^.]*"#)?;

        assert_eq!(pipe.pipeline[0].0.get_program(), "echo");
        assert_eq!(pipe.pipeline[0].0.get_args().collect::<Vec<&OsStr>>(), &["\"This is a test.\""]);
        assert_eq!(pipe.pipeline[1].0.get_program(), "grep");
        assert_eq!(pipe.pipeline[1].0.get_args().collect::<Vec<&OsStr>>(), &["-Eo", r"\w\w\sa[^.]*"]);

        Ok(())
    }
}