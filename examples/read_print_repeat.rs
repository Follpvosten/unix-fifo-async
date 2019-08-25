use async_std::{io, task};
use unix_fifo_async::NamedPipePath;

/// Repeatedly reads from `my_pipe` and prints the result.
/// Usage:
/// ```sh
/// $ cargo run --example read_print_repeat &
/// Waiting for message...
/// $ printf "something" > my_pipe
/// Received message: something
/// Waiting for message...
/// ```
fn main() -> io::Result<()> {
    task::block_on(async {
        let pipe = NamedPipePath::new("./my_pipe");
        let reader = pipe.open_read();
        loop {
            reader.ensure_pipe_exists().unwrap();
            println!("Waiting for message...");
            let next_msg = reader.read_string().await?;
            println!("Received message: {}", next_msg);
        }
    })
}
