use thiserror::Error;

#[derive(Error, Debug)]
pub enum APipeError {
    #[error("Tried to parse empty command string: {0}.")]
    MissingCommand(String),

    #[error("Previous command had not output to be captured.")]
    NoStdout,

    #[error("Failed to spawn command: {0}.")]
    FailedExecution(std::io::Error),

    #[error("Child process got terminated: {0}.")]
    TerminatedChildCommand(std::io::Error),

    #[error("Nothing to spawn in pipeline.")]
    EmptyPipe,
}
