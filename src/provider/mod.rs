mod dir;
mod file;
mod zpkg;

use anyhow::Error;
use std::env;
use std::path::PathBuf;
use crate::Phase;
use crate::action::{Action, ActionType, Dir, File, Zpkg};
use dir::*;
use file::*;
use zpkg::*;
use crate::zpkg::payload::{Reader, Writer};

#[derive(Clone)]
pub struct Options {
    pub target_path:  Option<PathBuf>,
    pub work_path:   Option<PathBuf>,

    pub debug:   bool,
    pub verbose: bool
}

impl Options {
    pub fn new() -> Options {
        Options {
            target_path: None,
            work_path: None,
            debug: false,
            verbose: false
        }
    }
}

pub trait Provider {
    fn realize(&self, opts: Options, phase: Phase, payload_reader: Option<&Reader>, payload_writer: Option<&mut Writer>) -> Result<Box<dyn Action>, Error>;
}

pub fn provider_for(action: Box<dyn Action>) -> Box<dyn Provider> {
    match action.type_name() {
        ActionType::Dir => Box::new(DirUnix::new(action.as_any().downcast_ref::<Dir>().unwrap().clone())),
        ActionType::File => Box::new(FileUnix::new(action.as_any().downcast_ref::<File>().unwrap().clone())),
        ActionType::Zpkg => Box::new(ZpkgDefault::new(action.as_any().downcast_ref::<Zpkg>().unwrap().clone()))
    }
}