//! Enables working with Unix named pipes (FIFOs) anywhere on the filesystem.
//!
//! It's important to note that currently, no locks on files are kept - the
//! `NamedPipe` struct only keeps the path in memory, the file could be
//! deleted at any point in time.
//!
//! # Example code
//!
//! Create a pipe, write to it in one async task and read from it in another:
//!
//! ```
//! # #![feature(async_await)]
//! # fn main() -> async_std::io::Result<()> { async_std::task::block_on(async {
//! #
//! use unix_fifo_async::NamedPipePath;
//! use async_std::task;
//!
//! // Create a new pipe at the given path
//! let pipe = NamedPipePath::new("./my_pipe");
//! // This creates the path if it doesn't exist; it may return a nix::Error
//! pipe.ensure_exists().unwrap();
//! // Create a writer and a reader on the path
//! let writer = pipe.open_write().await?;
//! let reader = pipe.open_read().await?;
//!
//! // Some data we can send over the pipe
//! let data_to_send = "Hello, pipes!";
//!
//! // Spawn two tasks, one for writing to and one for reading from the pipe.
//! let t1 = task::spawn(async move { writer.write_str(data_to_send).await });
//! let t2 = task::spawn(async move { reader.read_string().await });
//!
//! // `.await` both tasks and compare the result with the original
//! t1.await.unwrap();
//! let read_result = t2.await.unwrap();
//! assert_eq!(read_result, data_to_send);
//!
//! // Delete the pipe
//! pipe.delete().await?;
//! # Ok(())
//! # })}
//! ```
#![feature(async_await)]

mod named_pipe;

pub mod util;
pub use named_pipe::{NamedPipePath, NamedPipeReader, NamedPipeWriter};
pub use util::{create_pipe, remove_pipe};
