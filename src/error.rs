use std::error;
use std::fmt::Display;

#[derive(Debug)]
pub enum APipeError {
    Parser(String),
    ChildProcess(std::io::Error, &'static str),
    NoRunningProcesses,
}

impl Display for APipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            APipeError::Parser(ref cmd) => {
                write!(f, "Tried to parse empty command string: {}", cmd)
            }
            APipeError::ChildProcess(_, s) => write!(f, "{}", s),
            APipeError::NoRunningProcesses => write!(f, "No running processes."),
        }
    }
}

impl error::Error for APipeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            APipeError::ChildProcess(ref e, _) => Some(e),
            _ => None,
        }
    }
}
