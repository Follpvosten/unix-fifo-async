use async_std::{
    fs::File,
    io::{self, Read, Write},
};
use std::path::PathBuf;

/// Represents a path to a Unix named pipe (FIFO).
///
/// Provides convenience methods to create readers and writers, as well as an
/// easy way to ensure the pipe actually exists.
pub struct NamedPipePath {
    path: PathBuf,
}

impl NamedPipePath {
    /// Wraps a given path in a `NamedPipePath`.
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { path: path.into() }
    }
    /// Checks if the path exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
    // Ensures the path exists, creating a named pipe in its place if it doesn't.
    pub fn ensure_exists(&self) -> nix::Result<()> {
        if !self.exists() {
            crate::create_pipe(&self.path, None)
        } else {
            Ok(())
        }
    }
    /// Tries to delete the pipe from disk and consumes the `NamedPipe`.
    pub async fn delete(self) -> io::Result<()> {
        if self.path.exists() {
            crate::remove_pipe(&self.path).await
        } else {
            Ok(())
        }
    }

    /// Creates a reader for this named pipe.
    pub async fn open_read(&self) -> io::Result<NamedPipeReader> {
        NamedPipeReader::from_path(self).await
    }
    /// Creates a writer for this named pipe.
    pub async fn open_write(&self) -> io::Result<NamedPipeWriter> {
        NamedPipeWriter::from_path(self).await
    }
}

/// A convenience wrapper for reading from Unix named pipes.
///
/// Note: The actual idea of this struct is that it holds a File so it's
/// not possible to just delete the pipe it's reading from. However, at the
/// moment it doesn't seem to be possible to do that, as the `open`-call for
/// that file will block until something is written to the pipe, kind of
/// defeating the point of this wrapper. I'm currently considering workarounds,
/// like checking whether the pipe exists and recreating iton each call to a
/// read method.
pub struct NamedPipeReader {
    path: PathBuf,
}

impl NamedPipeReader {
    pub async fn from_path(source: &NamedPipePath) -> io::Result<Self> {
        let path = source.path.clone();
        //let handle = File::open(&path).await?;
        Ok(Self { path })
    }
    /// Reads all bytes from the pipe.
    /// This method is supposed to consume the reader in the future.
    pub async fn read(&self) -> io::Result<Vec<u8>> {
        let mut handle = File::open(&self.path).await?;
        let mut buf = Vec::new();
        handle.read_to_end(&mut buf).await?;
        Ok(buf)
    }
    // pub async fn read_reopen(&mut self) -> io::Result<Vec<u8>> {
    //     let mut buf = Vec::new();
    //     self.handle.read_to_end(&mut buf).await?;
    //     self.handle = File::open(&self.path).await?;
    //     Ok(buf)
    // }
    /// Reads a String from the pipe.
    /// This method is supposed to consume the reader in the future.
    pub async fn read_string(&self) -> io::Result<String> {
        let mut handle = File::open(&self.path).await?;
        let mut buf = String::new();
        handle.read_to_string(&mut buf).await?;
        Ok(buf)
    }
    // pub async fn read_string_reopen(&mut self) -> io::Result<String> {
    //     let mut buf = String::new();
    //     self.handle.read_to_string(&mut buf).await?;
    //     self.handle = File::open(&self.path).await?;
    //     Ok(buf)
    // }
}

/// A convenience wrapper for writing to Unix named pipes.
///
/// Note: The actual idea of this struct is that it holds a File so it's
/// not possible to just delete the pipe it's writing to. However, at the
/// moment it doesn't seem to be possible to do that, as the `open`-call for
/// that file will block until something is written to the pipe, kind of
/// defeating the point of this wrapper. I'm currently considering workarounds,
/// like checking whether the pipe exists and recreating it on each call to a
/// write method.
pub struct NamedPipeWriter {
    path: PathBuf,
    //handle: File,
}

impl NamedPipeWriter {
    async fn open(path: &PathBuf) -> io::Result<File> {
        //use async_std::os::unix::fs::OpenOptionsExt;
        //use nix::fcntl::OFlag;
        async_std::fs::OpenOptions::new()
            .write(true)
            .create(false)
            //.custom_flags(OFlag::O_NONBLOCK.bits())
            .open(path)
            .await
    }
    pub async fn from_path(source: &NamedPipePath) -> io::Result<Self> {
        let path = source.path.clone();
        //let handle = Self::open(&path).await?;
        Ok(Self { path })
    }
    /// Writes byte data to the pipe.
    /// This method is supposed to consume the Writer in the future.
    pub async fn write(&self, data: &[u8]) -> io::Result<()> {
        let mut handle = Self::open(&self.path).await?;
        handle.write_all(data).await
    }
    // pub async fn write_reopen(&mut self, data: &[u8]) -> io::Result<()> {
    //     let mut handle = Self::open(&self.path).await?;
    //     handle.write_all(data).await
    // }
    /// Writes &str data to the pipe.
    /// This method is supposed to consume the Writer in the future.
    pub async fn write_str(&self, data: &str) -> io::Result<()> {
        let mut handle = Self::open(&self.path).await?;
        handle.write_all(data.as_bytes()).await
    }
    // pub async fn write_str_reopen(&mut self, data: &str) -> io::Result<()> {
    //     let mut handle = Self::open(&self.path).await?;
    //     handle.write_all(data.as_bytes()).await
    // }
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    #[test]
    fn send_and_receive_threaded() {
        use std::thread;
        let pipe = super::NamedPipePath::new("./test_pipe_3");
        pipe.ensure_exists().unwrap();
        let writer = block_on(pipe.open_write()).unwrap();
        let reader = block_on(pipe.open_read()).unwrap();
        let data_to_send = "Hello pipe";
        let t_write = thread::spawn(move || block_on(writer.write_str(data_to_send)));
        let t_read = thread::spawn(move || block_on(reader.read_string()));
        t_write.join().unwrap().unwrap();
        let read_result = t_read.join().unwrap().unwrap();
        assert_eq!(read_result, data_to_send);
        block_on(pipe.delete()).unwrap();
    }
    #[test]
    fn send_and_receive_async() {
        block_on(async {
            use async_std::task;
            let pipe = super::NamedPipePath::new("./test_pipe_4");
            pipe.ensure_exists().unwrap();
            let writer = pipe.open_write().await.unwrap();
            let reader = pipe.open_read().await.unwrap();
            let data_to_send = "Hello pipe";
            let t1 = task::spawn(async move { writer.write_str(data_to_send).await });
            let t2 = task::spawn(async move { reader.read_string().await });
            t1.await.unwrap();
            let read_result = t2.await.unwrap();
            assert_eq!(read_result, data_to_send);
            pipe.delete().await.unwrap();
        });
    }
}
