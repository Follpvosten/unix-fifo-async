use async_std::{io, task};
use unix_fifo_async::NamedPipePath;

/// Reads something from the `reverse_me` pipe, reverses it and writes it back.
/// Usage:
/// ```sh
/// $ cargo run --example read_write_repeat
/// $ printf "some string" > reverse_me
/// $ cat reverse_me
/// gnirts emos
/// ```
fn main() -> io::Result<()> {
    task::block_on(async move {
        let pipe = NamedPipePath::new("./reverse_me");
        let writer = pipe.open_write();
        let reader = pipe.open_read();
        loop {
            pipe.ensure_exists().unwrap();
            println!("Waiting for message...");
            let msg = reader.read_string().await?;

            let answer = msg.chars().rev().collect::<String>() + "\n";
            pipe.ensure_exists().unwrap();
            println!("Received message, waiting for receiver...");
            writer.write_str(&answer).await?;
        }
    })
}
