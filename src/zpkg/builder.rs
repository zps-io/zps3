use std::env;
use std::fs::metadata;
use std::path::PathBuf;

use anyhow::*;
use event_emitter_rs::EventEmitter;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{Emitter, Package, Phase};
use crate::action::{Action, Dir, File, Manifest, Zpkg};
use crate::fs::Resolver;
use crate::provider::{Options, provider_for};
use crate::zpkg::header::{CompType, Header, HeaderV1, Version, HashMethod};
use crate::zpkg::payload;
use crate::zpkg::writer::Writer;
use std::borrow::BorrowMut;

const DEFAULT_ZPF_PATH: &str = "Zpkgfile";
const DEFAULT_TARGET_PATH: &str = "proto";

pub struct Builder {
    emitter: EventEmitter,

    compression: CompType,
    hash_method: HashMethod,
    provider_options: Options,
    version: Version,

    output_path: Option<PathBuf>,
    zpf_path: Option<PathBuf>,

    owner: Option<String>,
    group: Option<String>,

    secure: bool,
    restrict: bool,

    file_path: Option<PathBuf>,
    manifest: Option<Manifest>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            emitter: EventEmitter::new(),
            compression: CompType::ZSTD,
            hash_method: HashMethod::SHA3_256,
            provider_options: Options::new(),
            version: Version::V1,
            output_path: None,
            zpf_path: None,
            owner: None,
            group: None,
            secure: true,
            restrict: false,
            file_path: None,
            manifest: None,
        }
    }

    pub fn compression(&mut self, comp_type: CompType) -> &mut Builder {
        self.compression = comp_type;
        self
    }

    pub fn target(&mut self, path: String) -> &mut Builder {
        self.provider_options.target_path = Some(PathBuf::from(path));
        self
    }

    pub fn work(&mut self, path: String) -> &mut Builder {
        self.provider_options.work_path = Some(PathBuf::from(path));
        self
    }

    pub fn insecure(&mut self) -> &mut Builder {
        self.secure = false;
        self
    }

    pub fn restrict(&mut self) -> &mut Builder {
        self.restrict = true;
        self
    }

    pub fn owner(&mut self, owner: String) -> &mut Builder {
        self.owner = Some(owner);
        self
    }

    pub fn group(&mut self, group: String) -> &mut Builder {
        self.group = Some(group);
        self
    }

    pub fn debug(&mut self) -> &mut Builder {
        self.provider_options.debug = true;
        self
    }

    pub fn verbose(&mut self) -> &mut Builder {
        self.provider_options.verbose = true;
        self
    }

    pub fn version(&mut self, version: Version) -> &mut Builder {
        self.version = version;
        self
    }

    pub fn output(&mut self, path: String) -> &mut Builder {
        self.output_path = Some(PathBuf::from(path));
        self
    }

    pub fn zpf(&mut self, path: String) -> &mut Builder {
        self.zpf_path = Some(PathBuf::from(path));
        self
    }

    pub fn build(&mut self) -> Result<Manifest, Error> {
        self.set_paths()?;

        let writer = Writer::new();
        let mut payload = payload::Writer::new(self.compression, self.hash_method, self.provider_options.work_path.as_ref().unwrap().as_path())?;

        self.load_zpf()?;

        // This will restrict file system actions to those defined in the Zpkgfile
        if !self.restrict {
            self.resolve()?;
        }

        self.realize(payload.borrow_mut())?;
        self.validate()?;

        let manifest_bytes = match self.compression {
            CompType::ZSTD => zstd::block::compress(&self.manifest.as_ref().unwrap().to_json()?, 3)?
        };

        let header_bytes = match self.version {
            Version::V1 => {
                HeaderV1::new(self.compression, self.hash_method, manifest_bytes.len() as u32).to_vec()
            }
        };

        writer.write(self.file_path.as_ref().unwrap().clone().into_os_string().into_string().unwrap(), &header_bytes, &manifest_bytes, payload.file_path())?;

        std::fs::remove_file(payload.file_path());

        Ok(self.manifest.clone().unwrap())
    }

    fn set_paths(&mut self) -> Result<(), Error> {
        if self.output_path.is_none() {
            self.output_path = Some(env::current_dir()?);
        }

        if self.provider_options.work_path.is_none() {
            self.provider_options.work_path = Some(env::current_dir()?);
        }

        if self.zpf_path.is_none() {
            self.zpf_path = Some(env::current_dir()?);
            self.zpf_path.as_mut().unwrap().push(DEFAULT_ZPF_PATH);
        }

        if metadata(self.zpf_path.as_ref().unwrap())?.is_dir() {
            self.zpf_path.as_mut().unwrap().push(DEFAULT_ZPF_PATH);
        }

        if self.provider_options.target_path.is_none() {
            self.provider_options.target_path = self.zpf_path.clone();
            self.provider_options.target_path.as_mut().unwrap().pop();
            self.provider_options.target_path.as_mut().unwrap().push(DEFAULT_TARGET_PATH);
        }

        Ok(())
    }

    fn load_zpf(&mut self) -> Result<(), Error> {
        self.manifest = {
            let mut manifest = Manifest::new(Zpkg {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                publisher: "thing".to_string(),
                arch: "x86_64".to_string(),
                os: "darwin".to_string(),
                summary: "banan".to_string(),
                description: "narf".to_string(),
            });

            manifest.dirs = vec![
                Dir {
                    path: "nacho/taco".to_string(),
                    owner: "root".to_string(),
                    group: "root".to_string(),
                    mode: 0o0750,
                }
            ];

            manifest.files = vec![
                File {
                    path: "nacho/taco/sniffer".to_string(),
                    owner: "root".to_string(),
                    group: "root".to_string(),
                    mode: 0o0640,
                    digest: "".to_string(),
                    offset: 0,
                    csize: 0,
                    size: 0
                },
                File {
                    path: "nacho/oof.txt".to_string(),
                    owner: "root".to_string(),
                    group: "root".to_string(),
                    mode: 0o0640,
                    digest: "".to_string(),
                    offset: 0,
                    csize: 0,
                    size: 0
                }
            ];

            Some(manifest)
        };

        Ok(())
    }

    fn resolve(&mut self) -> Result<(), Error> {
        for action in Resolver::walk(self.provider_options.target_path.as_ref().unwrap(), self.secure)?.into_iter() {
            self.manifest.as_mut().unwrap().add(action);
        }
        Ok(())
    }

    // Realize may mutate actions, we reset them accordingly
    fn realize(&mut self, payload: &mut payload::Writer) -> Result<(), Error> {
        let mut actions : Vec<Box<dyn Action>> = Vec::new();

        for action in self.manifest.as_ref().unwrap().actions().into_iter() {
            let mut_action= provider_for(action).realize(self.provider_options.clone(), Phase::Package, None, Some(payload))?;

            println!("{}", mut_action.to_string());

            actions.push(mut_action);
        }

        self.manifest.as_mut().unwrap().set(actions);

        Ok(())
    }

    fn validate(&mut self) -> Result<(), Error> {
        self.file_path = self.output_path.clone();
        self.file_path.as_mut().unwrap().push(Package::from(self.manifest.as_ref().unwrap().clone())?.file_name());
        self.manifest.as_ref().unwrap().validate()?;

        Ok(())
    }
}

impl Emitter for Builder {
    fn on<F, T>(&mut self, event: &str, callback: F) -> String
        where
                for<'de> T: Deserialize<'de>,
                F: Fn(T) + 'static + Sync + Send
    {
        let id = self.emitter.on_limited(event, None, callback);
        return id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() -> Result<(), Error>{
        let manifest = Builder::new().build()?;
        assert_eq!(manifest.zpkg.name, "test".to_string());
        Ok(())
    }
}