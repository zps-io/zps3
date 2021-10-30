use crate::action::{Zpkg, Action};
use crate::provider::{Provider, Options};
use crate::Phase;
use anyhow::Error;
use crate::zpkg::payload::{Reader, Writer};

pub struct ZpkgDefault {
    pub action: Zpkg
}

impl ZpkgDefault {
    pub fn new(action: Zpkg) -> ZpkgDefault {
        ZpkgDefault{ action }
    }
}

impl Provider for ZpkgDefault {
    fn realize(&self, opts: Options, phase: Phase, payload_reader: Option<&Reader>, payload_writer: Option<&mut Writer>) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(self.action.clone()))
    }
}