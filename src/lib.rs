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
//!
//! use apipe::CommandPipe;
//!
//! let mut pipe = CommandPipe::try_from(r#"echo "This is a test." | grep -Eo \w\w\sa[^.]*"#)?;
//!
//! let output = pipe.spawn_with_output()?
//!                  .stdout();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Pipe Command Objects
//!
//! Another way is to create the individual Commands and then contruct a pipe from them:
//!
//! ```
//! # fn main() -> Result<(), apipe::error::APipeError> {
//!
//! use apipe::Command;
//!
//! let mut pipe = Command::parse_str(r#"echo "This is a test.""#)? | Command::parse_str(r#"grep -Eo \w\w\sa[^.]*"#)?;
//!                  
//! let output = pipe.spawn_with_output()?.stdout();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
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
//! Finally, there is a conventional builder syntax:
//!
//! ```
//! # fn main() -> Result<(), apipe::error::APipeError> {
//! use apipe::CommandPipe;
//!
//! let mut pipe = apipe::CommandPipe::new();
//!
//! pipe.add_command("echo")
//!     .arg("This is a test.")
//!     .add_command("grep")
//!     .arg("-Eo")
//!     .arg(r"\w\w\sa[^.]*")
//!     .spawn()?;
//!     
//! let output = pipe.output()?
//!                  .stdout();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
//!
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
