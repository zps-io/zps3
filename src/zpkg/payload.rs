use std::fs::File;
use anyhow::Error;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufWriter, Write, Seek, SeekFrom};
use std::io;
use crate::zpkg::header::{CompType, HashMethod};
use sha3::{Sha3_256, Digest};
use crate::io::MultiWriter;
use std::borrow::{BorrowMut, Borrow};

pub struct Reader {}

pub struct Writer {
    comp_type: CompType,
    hash_method: HashMethod,
    path: PathBuf,
    file: File
}

impl Writer {
    pub fn new(comp_type: CompType, hash_method: HashMethod, work_path: &Path) -> Result<Self, Error> {
        let mut xid = libxid::new_generator();
        let file_name = format!("{}.zpkgtmp", xid.new_id()?.encode());
        let path = work_path.clone().join(file_name);

        Ok(
            Self {
                comp_type,
                hash_method,
                path: path.clone(),
                file: File::create(path)?
            }
        )
    }

    pub fn file_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn put(&mut self, path: &Path) -> Result<(u64, u64, u64, String), Error> {
        // Allow creation of zero byte files from manifest
        if !path.exists() {
            return Ok((0, 0, 0, "".to_string()));
        }

        let input = File::open(path)?;
        let size = input.metadata()?.len();

        // Allow creation of zero byte files from filesystem
        if size == 0 {
            return Ok((0, 0, 0, "".to_string()));
        }

        let offset = self.file.seek(SeekFrom::Current(0))?;

        // TODO Figure out how to deal with this as Digest is not trait object safe
        let mut hasher = match self.hash_method {
            _ => Sha3_256::new()
        };

        let mut src = BufReader::new(input);
        let mut dst = MultiWriter::new(vec![Box::new(self.file.borrow_mut()), Box::new(hasher.borrow_mut())]);

        match self.comp_type {
            CompType::ZSTD => zstd::stream::copy_encode(src, dst, 3)?
        }

        let csize = self.file.seek(SeekFrom::Current(0))? - offset;

        Ok((offset, csize, size, format!{"{:x}",  hasher.finalize()}))
    }
}