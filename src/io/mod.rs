use std::io::Write;
use std::io;

pub struct MultiWriter<'a> {
    writers: Vec<Box<dyn Write + 'a>>
}

impl<'a> MultiWriter<'a> {
    pub fn new(writers: Vec<Box<dyn Write + 'a>>) -> MultiWriter {
        MultiWriter { writers }
    }
}

impl<'a> Write for MultiWriter<'a>  {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for writer in self.writers.iter_mut() {
            writer.write_all(buf)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        for writer in self.writers.iter_mut() {
            writer.flush()?;
        }

        Ok(())
    }
}