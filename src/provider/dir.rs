use crate::action::{Dir, Action};
use crate::provider::{Provider, Options};
use crate::Phase;
use anyhow::Error;
use crate::zpkg::payload::{Reader, Writer};

pub struct DirUnix {
    pub action: Dir
}

impl DirUnix {
    pub fn new(action: Dir) -> DirUnix {
        DirUnix{ action }
    }
}

impl Provider for DirUnix {
    fn realize(&self, opts: Options, phase: Phase, payload_reader: Option<&Reader>, payload_writer: Option<&mut Writer>) -> Result<Box<dyn Action>, Error> {
        match phase {
            _ => {}
        }

        Ok(Box::new(self.action.clone()))
    }
}