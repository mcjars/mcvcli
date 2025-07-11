use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub struct CountingReader<R: std::io::Read> {
    reader: R,
    count: Arc<AtomicUsize>,
}

impl<R: std::io::Read> CountingReader<R> {
    pub fn new(reader: R, count: Arc<AtomicUsize>) -> Self {
        Self { reader, count }
    }
}

impl<R: std::io::Read> std::io::Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        self.count.fetch_add(bytes_read, Ordering::SeqCst);

        Ok(bytes_read)
    }
}
