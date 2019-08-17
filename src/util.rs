use async_std::fs;
use nix::{sys::stat::Mode, NixPath};
use std::path::Path;

/// Attempt to create a new Unix named pipe/FIFO on disk.
pub fn create_pipe<P: ?Sized + NixPath>(path: &P, mode: Option<Mode>) -> nix::Result<()> {
    nix::unistd::mkfifo(path, mode.unwrap_or(Mode::from_bits_truncate(0o660)))
}

/// Attempt to delete a Unix named pipe/FIFO from disk.
pub async fn remove_pipe<P: AsRef<Path>>(path: P) -> async_std::io::Result<()> {
    fs::remove_file(&path).await
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    #[test]
    fn creation_deletion() {
        const FILENAME: &str = "./test_pipe_1";
        super::create_pipe(FILENAME, None).expect("Failed to create pipe");
        block_on(super::remove_pipe(FILENAME)).expect("Failed to remove pipe");
    }
    #[test]
    fn permissions() {
        use nix::sys::stat::{self, Mode};
        let path = std::path::Path::new("./test_pipe_2");
        let assert_stats_eq = |input: Option<Mode>| {
            super::create_pipe(path, input).expect("Failed to create pipe");
            let file_stat = stat::stat(path).expect("Failed to get file stat");
            let mode = Mode::from_bits_truncate(file_stat.st_mode);
            if let Some(new_mode) = input {
                assert_eq!(mode, new_mode);
            } else {
                assert_eq!(mode, Mode::from_bits_truncate(0o660));
            }
            block_on(super::remove_pipe(path)).expect("Failed to remove pipe");
        };
        // Defaults
        assert_stats_eq(None);
        // Custom mode
        assert_stats_eq(Some(Mode::from_bits_truncate(0o644)));
    }
}
