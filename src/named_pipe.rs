use async_std::{fs, io};
use std::path::PathBuf;

/// Represents a path to a Unix named pipe (FIFO).
///
/// Provides convenience methods to create readers and writers, as well as an
/// easy way to ensure the pipe actually exists.
#[derive(Clone)]
pub struct NamedPipePath {
    inner: PathBuf,
}

impl NamedPipePath {
    /// Wraps a given path in a `NamedPipePath`.
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { inner: path.into() }
    }
    /// Checks if the path exists.
    pub fn exists(&self) -> bool {
        self.inner.exists()
    }
    /// Ensures the path exists, creating a named pipe in its place if it doesn't.
    pub fn ensure_exists(&self) -> nix::Result<()> {
        if !self.exists() {
            crate::create_pipe(&self.inner, None)
        } else {
            Ok(())
        }
    }
    /// Tries to delete the pipe from disk and consumes the `NamedPipe`.
    pub async fn delete(self) -> io::Result<()> {
        if self.inner.exists() {
            crate::remove_pipe(&self.inner).await
        } else {
            Ok(())
        }
    }

    /// Creates a reader for this named pipe.
    pub fn open_read(&self) -> NamedPipeReader {
        NamedPipeReader::from_path(self)
    }
    /// Creates a writer for this named pipe.
    pub fn open_write(&self) -> NamedPipeWriter {
        NamedPipeWriter::from_path(self)
    }
}

/// A convenience wrapper for reading from Unix named pipes.
pub struct NamedPipeReader {
    path: NamedPipePath,
}

impl NamedPipeReader {
    /// Creates a new reader, cloning the given NamedPipePath.
    pub fn from_path(source: &NamedPipePath) -> Self {
        Self {
            path: source.clone(),
        }
    }
    /// Checks if the named pipe actually exists and tries to create it if it doesn't.
    pub fn ensure_pipe_exists(&self) -> nix::Result<&Self> {
        self.path.ensure_exists()?;
        Ok(self)
    }
    /// Reads all bytes from the pipe.
    /// The returned Future will resolve when something is written to the pipe.
    pub async fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path.inner).await
    }
    /// Reads a String from the pipe.
    /// The returned Future will resolve when something is written to the pipe.
    pub async fn read_string(&self) -> io::Result<String> {
        fs::read_to_string(&self.path.inner).await
    }
}

/// A convenience wrapper for writing to Unix named pipes.
pub struct NamedPipeWriter {
    path: NamedPipePath,
}

impl NamedPipeWriter {
    pub fn from_path(source: &NamedPipePath) -> Self {
        Self {
            path: source.clone(),
        }
    }
    /// Checks if the named pipe actually exists and tries to create it if it doesn't.
    pub fn ensure_pipe_exists(&self) -> nix::Result<&Self> {
        self.path.ensure_exists()?;
        Ok(self)
    }
    /// Writes byte data to the pipe.
    /// The returned Future will resolve when the bytes are read from the pipe.
    pub async fn write(&self, data: &[u8]) -> io::Result<()> {
        fs::write(&self.path.inner, data).await
    }
    /// Writes &str data to the pipe.
    /// The returned Future will resolve when the string is read from the pipe.
    pub async fn write_str(&self, data: &str) -> io::Result<()> {
        fs::write(&self.path.inner, data).await
    }
}

#[cfg(test)]
mod tests {
    use async_std::task::{self, block_on};
    #[test]
    fn write_and_read_threaded() {
        use std::thread;
        let pipe = super::NamedPipePath::new("./test_pipe_3");
        pipe.ensure_exists().unwrap();
        let writer = pipe.open_write();
        let reader = pipe.open_read();
        let data_to_send = "Hello pipe";
        let t_write = thread::spawn(move || block_on(writer.write_str(data_to_send)));
        let t_read = thread::spawn(move || block_on(reader.read_string()));
        t_write.join().unwrap().unwrap();
        let read_result = t_read.join().unwrap().unwrap();
        assert_eq!(read_result, data_to_send);
        block_on(pipe.delete()).unwrap();
    }
    #[test]
    fn write_and_read_async() {
        block_on(async {
            let pipe = super::NamedPipePath::new("./test_pipe_4");
            pipe.ensure_exists().unwrap();
            let writer = pipe.open_write();
            let reader = pipe.open_read();
            let data_to_send = "Hello pipe";
            let t1 = task::spawn(async move { writer.write_str(data_to_send).await });
            let t2 = task::spawn(async move { reader.read_string().await });
            t1.await.unwrap();
            let read_result = t2.await.unwrap();
            assert_eq!(read_result, data_to_send);
            pipe.delete().await.unwrap();
        });
    }
    #[test]
    fn ensure_on_write() {
        block_on(async {
            let pipe = super::NamedPipePath::new("./test_pipe_5");
            let writer = pipe.open_write();
            let reader = pipe.open_read();
            let data_to_send = "Hello pipe";
            let t1 = task::spawn(async move {
                writer
                    .ensure_pipe_exists()
                    .unwrap()
                    .write_str(data_to_send)
                    .await
            });
            let t2 = task::spawn(async move { reader.read_string().await });
            t1.await.unwrap();
            let read_result = t2.await.unwrap();
            assert_eq!(read_result, data_to_send);
            pipe.delete().await.unwrap();
        });
    }
    #[test]
    fn ensure_on_read() {
        block_on(async {
            let pipe = super::NamedPipePath::new("./test_pipe_6");
            let writer = pipe.open_write();
            let reader = pipe.open_read();
            let data_to_send = "Hello pipe";
            let t1 = task::spawn(async move { writer.write_str(data_to_send).await });
            let t2 =
                task::spawn(
                    async move { reader.ensure_pipe_exists().unwrap().read_string().await },
                );
            t1.await.unwrap();
            let read_result = t2.await.unwrap();
            assert_eq!(read_result, data_to_send);
            pipe.delete().await.unwrap();
        });
    }
}
