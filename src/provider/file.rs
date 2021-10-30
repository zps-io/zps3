use crate::action::{File, Action};
use crate::provider::{Provider, Options};
use crate::Phase;
use anyhow::Error;
use crate::zpkg::payload::{Reader, Writer};

pub struct FileUnix {
    pub action: File
}

impl FileUnix {
    pub fn new(action: File) -> Self {
        Self { action }
    }

    fn package(&self, opts: Options, payload_writer: &mut Writer) -> Result<Box<dyn Action>, Error> {
        let result = payload_writer.put(opts.target_path.unwrap().as_path().join(&self.action.path).as_path())?;

        let mut action = self.action.clone();
        action.offset = result.0;
        action.csize = result.1;
        action.size = result.2;
        action.digest = result.3;

        Ok(Box::new(action))
    }
}

impl Provider for FileUnix {
    fn realize(&self, opts: Options, phase: Phase, payload_reader: Option<&Reader>, payload_writer: Option<&mut Writer>) -> Result<Box<dyn Action>, Error> {
        match phase {
            Phase::Package => self.package(opts, payload_writer.unwrap()),
            _ =>  Ok(Box::new(self.action.clone()))
        }
    }
}