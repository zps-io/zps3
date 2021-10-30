use crate::zpkg::header::MAGIC;
use std::fs::File;
use std::io::{Write, BufReader};
use anyhow::Error;
use std::path::Path;
use std::borrow::BorrowMut;

pub struct Writer {}

impl Writer {
    pub fn new() -> Writer {
        Writer {}
    }

    pub fn write(&self, file_path: String, header_bytes: &[u8], manifest_bytes: &[u8], payload: &Path) -> Result<(), Error> {
        let mut buffer = File::create(file_path)?;

        // Write magic
        buffer.write_all(MAGIC)?;

        // Write header
        buffer.write_all(&header_bytes)?;

        // Write manifest
        buffer.write_all(&manifest_bytes)?;

        // Copy payload to file
        let pl = File::open(payload)?;
        let mut reader = BufReader::new(pl);

        std::io::copy(reader.borrow_mut(), buffer.borrow_mut())?;

        Ok(())
    }
}