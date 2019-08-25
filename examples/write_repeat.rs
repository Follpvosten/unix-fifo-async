use async_std::{io, task};
use unix_fifo_async::NamedPipePath;

/// Repeatedly writes the given string (or "Hello world!") to `my_pipe`.
/// Usage:
/// ```sh
/// $ cargo run --example write_repeat -- blub
/// $ cat reverse_me
/// blub
/// $ cat reverse_me
/// blub
/// ```
fn main() -> io::Result<()> {
    let text = std::env::args()
        .nth(1)
        .unwrap_or("Hello world!".to_string())
        + "\n";
    println!("Writing String: {}", &text);
    task::block_on(async move {
        let pipe = NamedPipePath::new("./my_pipe");
        let writer = pipe.open_write();
        loop {
            writer.ensure_pipe_exists().unwrap();
            println!("Waiting for receiver...");
            writer.write_str(&text).await?;
        }
    })
}
