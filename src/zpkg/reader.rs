use std::path::{Path, PathBuf};
use crate::zpkg::header::{Header, Version, HeaderV1, CompType, HashMethod};
use crate::action::Manifest;
use anyhow::Error;
use std::fs::File;
use std::io::BufReader;
use byteorder::{ReadBytesExt, LittleEndian};
use url::quirks::hash;
use std::hash::Hash;

pub struct Reader {
    path: PathBuf,
    work_path: PathBuf,

    pub header: Option<Box<dyn Header>>,
    pub manifest: Option<Manifest>
}

impl Reader {
    pub fn new(path: &Path, work_path: &Path) -> Self {
        Self {
            path: PathBuf::from(path),
            work_path: PathBuf::from(work_path),
            header: None,
            manifest: None
        }
    }

    pub fn read (&mut self) -> Result<(), Error> {
        let file = File::open(self.path.as_path())?;
        let mut reader = BufReader::new(file);

        let _ = reader.read_u8()?;
        let comp_type = reader.read_u8()?.into();
        let hash_method = reader.read_u8()?.into();
        let manifest_len = reader.read_u32::<LittleEndian>()?;

        self.header = Some(Box::new(HeaderV1::new(comp_type, hash_method, manifest_len)));

        Ok(())

    }
}