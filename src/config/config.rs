use std::path::{Path, PathBuf};
use crate::platform::{OSArch, OS, Arch};
use crate::config::path::*;

pub struct Config {
    os_arch: OSArch,
    tree: PathBuf
}

impl Config {
    pub fn new() -> Config {
        return Config {
            os_arch: OSArch::from_current(),
            tree: resolve_tree()
        }
    }
    
    pub fn for_tree(tree: &Path) -> Config {
        return Config {
            os_arch: osarch_from_config(tree),
            tree: tree.to_path_buf()
        }
    }

    pub fn arch(&self) -> Arch {
        self.os_arch.arch()
    }

    pub fn os(&self) -> OS {
        self.os_arch.os()
    }

    pub fn tree(&self) -> PathBuf {
        self.tree.clone()
    }

    pub fn cache_path(&self) -> PathBuf {
        Path::join(self.tree().as_path(), TMP)
    }

    pub fn config_path(&self) -> PathBuf {
        Path::join(self.tree().as_path(), ETC)
    }

    pub fn data_path(&self) -> PathBuf {
        Path::join(self.tree().as_path(), DATA)
    }

    pub fn tmp_path(&self) -> PathBuf {
        Path::join(self.tree().as_path(), TMP)
    }
}

fn osarch_from_config(tree: &Path) -> OSArch {
    // TODO
    // load config from tree
    OSArch::from_current()
}