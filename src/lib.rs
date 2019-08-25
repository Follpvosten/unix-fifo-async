//! Eases working with Unix named pipes (FIFOs) anywhere on the filesystem.
//!
//! Because of the way this works currently, there's no real way to get a
//! lock on the pipe, but there are convenience methods on both `NamedPipePath`
//! and `NamedPipeReader`/`NamedPipeWriter` to ensure the pipe exists.
//!
//! # Example code
//!
//! Create a pipe, write to it in one async task and read from it in another:
//!
//! ```
//! # fn main() -> async_std::io::Result<()> { async_std::task::block_on(async {
//! #
//! use unix_fifo_async::NamedPipePath;
//! use async_std::task;
//!
//! // Create a new pipe at the given path
//! let pipe = NamedPipePath::new("./my_pipe");
//! // This creates the path if it doesn't exist; it may return a nix::Error
//! // You can also use the `ensure_pipe_exists` convenience function on
//! // readers/writers, but calling it on both at the same time results
//! // in a race condition so it can never succeed.
//! pipe.ensure_exists().unwrap();
//! // Create a writer and a reader on the path
//! let writer = pipe.open_write();
//! let reader = pipe.open_read();
//!
//! // Some data we can send over the pipe
//! let data_to_send = "Hello, pipes!";
//!
//! // Spawn two tasks, one for writing to and one for reading from the pipe.
//! let t1 = task::spawn(async move { writer.write_str(data_to_send).await });
//! let t2 = task::spawn(async move { reader.read_string().await });
//!
//! // `.await` both tasks and compare the result with the original
//! t1.await?;
//! let read_result = t2.await?;
//! assert_eq!(read_result, data_to_send);
//!
//! // Delete the pipe
//! pipe.delete().await?;
//! # Ok(())
//! # })}
//! ```
//!
//! Note that in practice, you'll probably want to read the pipe from a different
//! process or have it read by an entirely different program.

mod named_pipe;

pub mod util;
pub use named_pipe::{NamedPipePath, NamedPipeReader, NamedPipeWriter};
pub use util::{create_pipe, remove_pipe};
