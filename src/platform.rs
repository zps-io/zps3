/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use std::fmt;
use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

#[derive(Copy, Clone, Debug, PartialEq, EnumIter, EnumString)]
pub enum OS {
    #[strum(serialize = "any")]
    Any,
    #[strum(serialize = "darwin")]
    Darwin,
    #[strum(serialize = "linux")]
    Linux,
}

#[derive(Copy, Clone, Debug, PartialEq, EnumIter, EnumString)]
pub enum Arch {
    #[strum(serialize = "any")]
    Any,
    #[strum(serialize = "arm64")]
    Arm64,
    #[strum(serialize = "x86_64")]
    X8664,
}

#[derive(Clone, Copy, Debug)]
pub struct OSArch {
    os: OS,
    arch: Arch,
}

impl Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OS::Any => write!(f, "{}", String::from("any")),
            OS::Darwin => write!(f, "{}", String::from("darwin")),
            OS::Linux => write!(f, "{}", String::from("linux")),
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arch::Any => write!(f, "{}", String::from("any")),
            Arch::Arm64 => write!(f, "{}", String::from("arm64")),
            Arch::X8664 => write!(f, "{}", String::from("x86_64")),
        }
    }
}

impl Display for OSArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.os.to_string(), self.arch.to_string())
    }
}

impl OSArch {
    pub fn new(os: OS, arch: Arch) -> OSArch {
        OSArch { os, arch }
    }

    pub fn from_current() -> OSArch {
        let os = match std::env::consts::OS {
            "linux" => OS::Linux,
            "macos" => OS::Darwin,
            _ => panic!("Unsupported OS"),
        };

        let arch = match std::env::consts::ARCH {
            "aarch64" => Arch::Arm64,
            "x86_64" => Arch::X8664,
            _ => panic!("Unsupported Arch"),
        };

        OSArch { os, arch }
    }

    pub fn platforms() -> Vec<OSArch> {
        let mut platforms: Vec<OSArch> = Vec::new();

        for os in OS::iter() {
            for arch in Arch::iter() {
                platforms.push(OSArch { os, arch })
            }
        }

        platforms
    }

    pub fn expand(&self) -> Vec<OSArch> {
        let mut platforms: Vec<OSArch> = Vec::new();

        platforms.push(self.clone());
        platforms.push(OSArch {
            os: self.os,
            arch: Arch::Any,
        });
        platforms.push(OSArch {
            os: OS::Any,
            arch: Arch::Any,
        });
        platforms.push(OSArch {
            os: OS::Any,
            arch: self.arch,
        });

        platforms
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn os(&self) -> OS {
        self.os
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(
            OSArch {
                os: OS::Darwin,
                arch: Arch::X8664
            }
            .to_string(),
            String::from("darwin-x86_64")
        )
    }

    #[test]
    fn test_expand() {
        let current = OSArch::from_current();
        let expanded = current.expand();

        assert_eq!(expanded.len(), 4)
    }
}
