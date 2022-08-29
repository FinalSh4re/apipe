//! A simple annonymous UNIX pipe type.
//!
//! ## Usage
//!
//! ### try_from(&str)
//!
//! The probably easiest way to create a pipe is by parsing a command string:
//!
//! ```
//! # fn main() -> Result<(), apipe::error::APipeError> {
//! use apipe::CommandPipe;
//!
//! let mut pipe = CommandPipe::try_from(r#"echo "This is a test." | grep -Eo \w\w\sa[^.]*"#)?;
//!
//! let output = pipe.spawn_with_output()?;
//!     
//! assert_eq!(output.stdout(), "is a test\n".as_bytes());
//!
//! # Ok(())
//! # }
//! ```
//! This requires the `parser` feature to be enabled.
//!
//! ### Pipe Command Objects
//!
//! Create the individual Commands and then contruct a pipe from them:
//!
//! ```
//! # fn main() -> Result<(), apipe::error::APipeError> {
//! use apipe::Command;
//!
//! let mut pipe = Command::parse_str(r#"echo "This is a test.""#)?
//!              | Command::parse_str(r#"grep -Eo \w\w\sa[^.]*"#)?;
//!
//! // or:
//!
//! let mut pipe = Command::new("echo").arg("This is a test.")
//!              | Command::new("grep").args(&["-Eo", r"\w\w\sa[^.]*"]);
//!                  
//! let output = pipe.spawn_with_output()?;
//!     
//! assert_eq!(output.stdout(), "is a test\n".as_bytes());
//!
//! # Ok(())
//! # }
//! ```
//!
//! [Command]s can also be constructed manually if you want:
//!
//! ```
//! # use apipe::Command;
//! let mut command = Command::new("ls").arg("-la");
//! ```
//!
//! ### Builder
//!
//! There is also a conventional builder syntax:
//!
//! ```
//! # fn main() -> Result<(), apipe::error::APipeError> {
//! use apipe::CommandPipe;
//!
//! let output = apipe::CommandPipe::new()
//!     .add_command("echo")
//!     .arg("This is a test.")
//!     .add_command("grep")
//!     .args(&["-Eo", r"\w\w\sa[^.]*"])
//!     .spawn_with_output()?;
//!
//! assert_eq!(output.stdout(), "is a test\n".as_bytes());
//! # Ok(())
//! # }
//! ```

pub mod cmd;
pub mod error;
pub mod output;
pub mod pipe;

#[doc(inline)]
pub use cmd::Command;
#[doc(inline)]
pub use error::APipeError;
#[doc(inline)]
pub use pipe::CommandPipe;
