use std::process;

#[derive(Debug)]
pub struct Output(process::Output);

impl From<process::Output> for Output {
    fn from(command: process::Output) -> Self {
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
