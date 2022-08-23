//! A simple annonymous UNIX pipe type.
//!
//! ## Usage
//!
//! ```
//! let mut pipe = apipe::CommandPipe::new();
//!
//! pipe.add_command("echo")
//!     .arg("This is a test.")
//!     .add_command("grep")
//!     .arg("-Eo")
//!     .arg(r"\w\w\sa[^.]*")
//!     .spawn()
//!     .expect("Failed to spawn pipe.");
//!     
//! let output = pipe.output().unwrap().stdout();
//!     
//! assert_eq!(&String::from_utf8_lossy(output), "is a test\n");
//! ```

pub mod cmd;
pub mod pipe;
pub mod output;

pub use cmd::Command;
pub use pipe::CommandPipe;


