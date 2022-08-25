use std::process;

/// Provides a thin wrapper around [std::process::Output]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output(process::Output);

impl From<process::Output> for Output {
    fn from(command: process::Output) -> Self {
        Output(command)
    }
}

impl Output {
    /// See the `status` field of [std::process::Output]
    pub fn status_code(&self) -> Option<i32> {
        self.0.status.code()
    }
    /// See the `stdout` field of [std::process::Output]
    pub fn stdout(&self) -> &[u8] {
        self.0.stdout.as_slice()
    }
    /// See the `stderr` field of [std::process::Output]
    pub fn stderr(&self) -> &[u8] {
        self.0.stderr.as_slice()
    }
}
