/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use anyhow::*;
use chrono::{DateTime, TimeZone, Utc};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;

trait Exq {
    fn exq(&self, other: &Self) -> bool;
}

#[derive(Debug, Clone)]
pub struct Version {
    semver: semver::Version,
    time: Option<DateTime<Utc>>,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Version {
        Version {
            semver: semver::Version::new(major, minor, patch),
            time: Some(Utc::now()),
        }
    }

    pub fn from<S: Into<String>>(version: S) -> Result<Version, Error> {
        let version_into = version.into();

        let parts: Vec<&str> = version_into.split(":").collect();

        if parts.len() < 1 {
            return Err(anyhow!("invalid version string: {}", version_into));
        }

        let semver = semver::Version::parse(parts[0])?;

        let mut time = None;
        if parts.len() == 2 {
            time = Some(Utc.datetime_from_str(parts[1], "%Y%m%dT%H%M%SZ")?);
        }

        Ok(Version { semver, time })
    }

    pub fn to_string(&self) -> String {
        match self.time {
            None => format!(
                "{}",
                self.semver.to_string()
            ),
            Some(t) => format!(
                "{}:{}",
                self.semver.to_string(),
                t.format("%Y%m%dT%H%M%SZ")
            )
        }
    }
}

impl Eq for Version {}

impl Exq for Version {
    fn exq(&self, other: &Self) -> bool {
        self.semver == other.semver && self.time == other.time
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.semver > other.semver {
            return Ordering::Greater
        }

        if self.semver < other.semver {
            return Ordering::Less
        }

        if self.time > other.time {
            return Ordering::Greater
        }

        if self.time < other.time {
            return Ordering::Less
        }

        Ordering::Equal
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.semver == other.semver
    }
}

enum Comparator {
    ANY,
    GTE,
    LTE,
    EQ,
    EXQ
}

enum Method {
    Depends,
    Provides,
    Conflicts
}

impl Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Comparator::ANY => write!(f, "{}", String::from("ANY")),
            Comparator::GTE => write!(f, "{}", String::from("GTE")),
            Comparator::LTE => write!(f, "{}", String::from("LTE")),
            Comparator::EQ => write!(f, "{}", String::from("EQ")),
            Comparator::EXQ => write!(f, "{}", String::from("EXQ"))
        }
    }
}

struct Requirement {
    name: String,
    method: Method,
    comparator: Comparator,
    version: Option<Version>
}

impl Requirement {
    pub fn new(name: String, method: Method, comparator: Comparator, version: Version) -> Requirement {
        Requirement {
            name,
            method,
            comparator,
            version: Some(version)
        }
    }

    pub fn from_simple<S: Into<String>>(requirement: S) -> Result<Requirement, Error> {
        let requirement_into = requirement.into();

        let parts: Vec<&str> = requirement_into.split("@").collect();

        if parts.len() < 2 {
            return Ok(
                Requirement{
                    name: requirement_into,
                    method: Method::Depends,
                    comparator: Comparator::ANY,
                    version: None
                }
            )
        }

        let version = Version::from(parts[1])?;

        match version.time {
            None => Ok(Requirement{
                name: String::from(parts[0]),
                method: Method::Depends,
                comparator: Comparator::EQ,
                version: Some(version)
            }),
            Some(_) => Ok(Requirement{
                name: String::from(parts[0]),
                method: Method::Depends,
                comparator: Comparator::EXQ,
                version: Some(version)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_long() -> Result<(), Error> {
        let version_string = "3.2.1:20200415T194203Z";
        let version = Version::from(version_string);

        assert_eq!(version?.to_string(), version_string);
        Ok(())
    }

    #[test]
    fn test_version_from_short() -> Result<(), Error> {
        let version_string = "3.2.1";
        let version = Version::from(version_string);

        assert_eq!(version?.semver.to_string(), version_string);
        Ok(())
    }

    #[test]
    fn test_version_partialeq() -> Result<(), Error> {
        let version_string = "3.2.1:20200415T194203Z";

        let version = Version::from(version_string)?;
        let same_version = Version::from(version_string)?;

        let other_version = Version::from("3.3.1:20200415T194203Z")?;

        assert_eq!(version, same_version);
        assert_ne!(version, other_version);
        Ok(())
    }

    #[test]
    fn test_version_exq() -> Result<(), Error> {
        let version_string = "3.2.1:20200415T194203Z";

        let version = Version::from(version_string)?;
        let same_version = Version::from(version_string)?;

        let other_version = Version::from("3.2.1:20200515T194203Z")?;

        assert_eq!(true, version.exq(&same_version));
        assert_eq!(false, version.exq(&other_version));
        Ok(())
    }

    #[test]
    fn test_version_ordering() -> Result<(), Error> {
        let v1 =  Version::from("3.2.1:20200515T194203Z")?;
        let v2 = Version::from("3.2.1:20200415T194203Z")?;

        let v3 = Version::from("3.0.1:20200415T194203Z")?;
        let v4 = Version::from("3.2.1:20200415T194203Z")?;

        assert_eq!(v1, v1.clone().max(v2));
        assert_eq!(v3, v3.clone().min(v4));
        Ok(())
    }

    #[test]
    fn test_requirement_from_simple() -> Result<(), Error> {
        let long = "zps@3.2.1:20200415T194203Z";
        let short = "zps@3.2.1";
        let name = "zps";

        let req_long = Requirement::from_simple(long)?;
        assert_eq!("zps", req_long.name);
        assert_eq!(Comparator::EXQ.to_string(), req_long.comparator.to_string());
        assert_eq!(req_long.version.unwrap().to_string(), "3.2.1:20200415T194203Z");

        let req_short = Requirement::from_simple(short)?;
        assert_eq!("zps", req_short.name);
        assert_eq!(Comparator::EQ.to_string(), req_short.comparator.to_string());
        assert_eq!(req_short.version.unwrap().to_string(), "3.2.1");

        let req_name = Requirement::from_simple(name)?;
        assert_eq!("zps", req_name.name);
        assert_eq!(Comparator::ANY.to_string(), req_name.comparator.to_string());
        assert_eq!(req_name.version, None);

        Ok(())
    }
}