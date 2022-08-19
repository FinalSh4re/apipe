use anyhow::{Context, Result};
use std::process::{Command, Output, Stdio};

pub struct CommandPipe(Vec<Command>);

impl CommandPipe {
    pub fn new() -> Self {
        CommandPipe(Vec::new())
    }

    pub fn add_command(&mut self, c: &str) -> &mut Self {
        let command = Command::new(c);
        self.0.push(command);

        self
    }

    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.0
            .last_mut()
            .expect("No Command in pipe to add args to.")
            .arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.0
            .last_mut()
            .expect("No Command in pipe to add args to.")
            .args(args);
        self
    }

    pub fn spawn(mut self) -> Result<Output> {
        let mut commands = self.0.iter_mut();
        let mut child = commands
            .next()
            .context("The pipe seems to be empty, no Commands to spawn.")?
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child
            .stdout
            .take()
            .context("Couldn't read stdout of previous command.")?;

        for proc in commands {
            child = proc
                .stdin(Stdio::from(stdin))
                .stdout(Stdio::piped())
                .spawn()?;

            child.wait().with_context(|| {
                format!(
                    "Child process '{}' exited with error code.",
                    proc.get_program().to_string_lossy()
                )
            })?;

            stdin = child
                .stdout
                .take()
                .context("Couldn't read stdout of previous command.")?;
        }

        child.stdout = Some(stdin);

        let output = child
            .wait_with_output()
            .context("Failed to wait for process.")?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").arg("-la").arg("~/Documents");

        let args: Vec<&std::ffi::OsStr> = pipe.0[0].get_args().collect();

        assert_eq!(args, &["-la", "~/Documents"])
    }

    #[test]
    fn test_args() {
        let mut pipe = CommandPipe::new();

        pipe.add_command("ls").args(vec!["-la", "~/Documents"]);

        let args: Vec<&std::ffi::OsStr> = pipe.0[0].get_args().collect();

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
