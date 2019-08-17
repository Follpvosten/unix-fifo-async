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
//! # fn main() { async_std::task::block_on(async {
//! #
//! use unix_fifo_async::NamedPipe;
//! use async_std::task;
//!
//! // Create a new pipe at the given path
//! let read_pipe = NamedPipe::create_new("./my_pipe".into()).unwrap();
//! // Clone the pipe
//! let write_pipe = read_pipe.clone();
//!
//! // Some data we can send over the pipe
//! let data_to_send = "Hello, pipes!";
//!
//! // Spawn two tasks, one for writing to and one for reading from the pipe.
//! let t1 = task::spawn(async move { write_pipe.write_str(data_to_send).await });
//! let t2 = task::spawn(async move { read_pipe.read_string().await });
//!
//! // `.await` both tasks and compare the result with the original
//! t1.await.unwrap();
//! let read_result = t2.await.unwrap();
//! assert_eq!(read_result, data_to_send);
//!
//! // Delete the pipe we don't need anymore
//! unix_fifo_async::remove_pipe("./my_pipe").await.unwrap();
//! #
//! # })}
//! ```
#![feature(async_await)]

mod named_pipe;
mod util;

pub use named_pipe::NamedPipe;
pub use util::{create_pipe, remove_pipe};
