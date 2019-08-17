use async_std::{fs, io};
use std::path::PathBuf;

/// Represents a Unix named pipe (FIFO).
///
/// No locks are held on the file itself; it could be deleted at any point in time.
#[derive(Clone)]
pub struct NamedPipe {
    path: PathBuf,
}

impl NamedPipe {
    /// Creates a new `NamedPipe` pointing at the given file.
    ///
    /// This method checks if the file exists and returns an error message if it doesn't.
    pub fn from_existing(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            Ok(Self { path })
        } else {
            Err("Pipe does not exist!".into())
        }
    }
    /// Creates a new named pipe on Disk and returns a `NamedPipe` pointing to it.
    ///
    /// If the pipe could not be created for some reason, an error is returned.
    pub fn create_new(path: PathBuf) -> nix::Result<Self> {
        crate::create_pipe(&path, None)?;
        Ok(Self { path })
    }
    /// Crates a new named pipe at the given path, with the given mode.
    ///
    /// If the pipe could not be created for some reason, an error is returned.
    pub fn create_with_mode(path: PathBuf, mode: nix::sys::stat::Mode) -> nix::Result<Self> {
        crate::create_pipe(&path, Some(mode))?;
        Ok(Self { path })
    }

    /// Opens the pipe for reading, reads until EOF and returns the result.
    pub async fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path).await
    }
    /// Opens the pipe for reading, reads until EOF and returns the result as a String.
    pub async fn read_string(&self) -> io::Result<String> {
        fs::read_to_string(&self.path).await
    }
    /// Writes the given bytes to the pipe.
    pub async fn write(&self, data: &[u8]) -> io::Result<()> {
        fs::write(&self.path, data).await
    }
    /// Writes the given `&str` to the pipe.
    pub async fn write_str(&self, data: &str) -> io::Result<()> {
        fs::write(&self.path, data).await
    }

    /// Tries to delete the pipe from disk and the `NamedPipe` from memory.
    pub async fn delete(self) -> io::Result<()> {
        crate::remove_pipe(self.path).await
    }
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    #[test]
    fn send_and_receive_threaded() {
        use std::thread;
        let pipe_write = super::NamedPipe::create_new("./test_pipe_3".into()).unwrap();
        let pipe_read = pipe_write.clone();
        let data_to_send = "Hello pipe";
        let t_write = thread::spawn(move || block_on(pipe_write.write_str(data_to_send)));
        let t_read = thread::spawn(move || block_on(pipe_read.read_string()));
        t_write.join().unwrap().unwrap();
        let read_result = t_read.join().unwrap().unwrap();
        assert_eq!(read_result, data_to_send);
        std::fs::remove_file("./test_pipe_3").unwrap();
    }
    #[test]
    fn send_and_receive_async() {
        block_on(async {
            use async_std::task;
            let pipe_write = super::NamedPipe::create_new("./test_pipe_4".into()).unwrap();
            let pipe_read = pipe_write.clone();
            let data_to_send = "Hello pipe";
            let t1 = task::spawn(async move { pipe_write.write_str(data_to_send).await });
            let t2 = task::spawn(async move { pipe_read.read_string().await });
            t1.await.unwrap();
            let read_result = t2.await.unwrap();
            assert_eq!(read_result, data_to_send);
            crate::remove_pipe("./test_pipe_4").await.unwrap();
        });
    }
}
