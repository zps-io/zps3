use std::path::Path;
use crate::action::{Action, Dir, File};
use anyhow::Error;
use walkdir::WalkDir;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use users::{get_user_by_uid, get_group_by_gid, User};

pub struct Resolver {}

impl Resolver {
    pub fn walk(target: &Path, secure: bool) -> Result<Vec<Box<dyn Action>>, Error> {
        let mut actions : Vec<Box<dyn Action>> = Vec::new();

        for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
            if entry.path() == target {
                continue
            }

            let path = entry.path().strip_prefix(target)?;
            let meta = entry.metadata()?;

            let mut owner_str = "root".to_string();
            let mut group_str = "root".to_string();

            if !secure {
                let owner = get_user_by_uid(meta.uid());
                let group = get_group_by_gid(meta.gid());

                if owner.is_some() {
                    owner_str = owner.unwrap().name().to_str().unwrap().to_string();
                }

                if group.is_some() {
                    group_str = group.unwrap().name().to_str().unwrap().to_string();
                }
            }

            if meta.is_dir() {
                actions.push(Box::new(
                    Dir {
                        path: path.to_str().unwrap().to_string(),
                        owner: owner_str,
                        group: group_str,
                        mode: meta.permissions().mode() & 0o7777
                    }
                ));

                continue
            }

            if meta.file_type().is_symlink() {
                continue
            }

            if meta.is_file() {
                actions.push(Box::new(
                    File {
                        path: path.to_str().unwrap().to_string(),
                        owner: owner_str,
                        group: group_str,
                        mode: meta.permissions().mode() & 0o7777,

                        offset: 0,
                        csize: 0,
                        digest: "".to_string(),
                        size: 0
                    }
                ));
            }
        }

        Ok(actions)
    }
}