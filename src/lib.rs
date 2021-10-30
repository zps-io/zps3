/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

mod action;
pub mod app;
pub mod config;
pub mod console;
mod db;
mod platform;
mod provider;
pub mod zpkg;
pub mod fs;
pub mod io;

use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::*;
use chrono::{DateTime, TimeZone, Utc};
use url::Url;

use platform::*;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use serde::Deserialize;
use crate::action::Manifest;

pub trait Emitter {
    fn on<F, T>(&mut self, event: &str, callback: F) -> String
        where
                for<'de> T: Deserialize<'de>,
                F: Fn(T) + 'static + Sync + Send;
}

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
        } else {
            time = Some(Utc::now())
        }

        Ok(Version { semver, time })
    }

    pub fn to_string(&self) -> String {
        match self.time {
            None => format!("{}", self.semver.to_string()),
            Some(t) => format!("{}:{}", self.semver.to_string(), t.format("%Y%m%dT%H%M%SZ")),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.time {
            None => write!(f, "{}", self.semver.to_string()),
            Some(t) => write!(
                f,
                "{}:{}",
                self.semver.to_string(),
                t.format("%Y%m%dT%H%M%SZ")
            ),
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
            return Ordering::Greater;
        }

        if self.semver < other.semver {
            return Ordering::Less;
        }

        if self.time > other.time {
            return Ordering::Greater;
        }

        if self.time < other.time {
            return Ordering::Less;
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

#[derive(Clone, PartialEq)]
enum Comparator {
    ANY,
    GTE,
    LTE,
    EQ,
    EXQ,
}

impl Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Comparator::ANY => write!(f, "{}", String::from("ANY")),
            Comparator::GTE => write!(f, "{}", String::from("GTE")),
            Comparator::LTE => write!(f, "{}", String::from("LTE")),
            Comparator::EQ => write!(f, "{}", String::from("EQ")),
            Comparator::EXQ => write!(f, "{}", String::from("EXQ")),
        }
    }
}

#[derive(Clone, PartialEq)]
enum RequirementMethod {
    Depends,
    Provides,
    Conflicts,
}

#[derive(Clone)]
struct Requirement {
    name: String,
    method: RequirementMethod,
    comparator: Comparator,
    version: Option<Version>,
}

impl Requirement {
    pub fn new(
        name: String,
        method: RequirementMethod,
        comparator: Comparator,
        version: Option<Version>,
    ) -> Requirement {
        Requirement {
            name,
            method,
            comparator,
            version,
        }
    }

    pub fn from_simple<S: Into<String>>(requirement: S) -> Result<Requirement, Error> {
        let requirement_into = requirement.into();

        let parts: Vec<&str> = requirement_into.split("@").collect();

        if parts.len() < 2 {
            return Ok(Requirement {
                name: requirement_into,
                method: RequirementMethod::Depends,
                comparator: Comparator::ANY,
                version: None,
            });
        }

        let version = Version::from(parts[1])?;

        match version.time {
            None => Ok(Requirement {
                name: String::from(parts[0]),
                method: RequirementMethod::Depends,
                comparator: Comparator::EQ,
                version: Some(version),
            }),
            Some(_) => Ok(Requirement {
                name: String::from(parts[0]),
                method: RequirementMethod::Depends,
                comparator: Comparator::EXQ,
                version: Some(version),
            }),
        }
    }
}

#[derive(PartialEq, Debug)]
enum OperationMethod {
    Install,
    Remove,
    NoOp,
}

impl Display for OperationMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OperationMethod::Install => write!(f, "{}", String::from("install")),
            OperationMethod::Remove => write!(f, "{}", String::from("remove")),
            OperationMethod::NoOp => write!(f, "{}", String::from("noop")),
        }
    }
}

// TODO requires graph node
struct Operation {
    method: OperationMethod,
    package: Package,
}

impl Operation {
    pub fn new(method: OperationMethod, package: Package) -> Operation {
        Operation { method, package }
    }
}

#[derive(PartialEq, Debug)]
enum RequestMethod {
    Install,
    Remove,
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestMethod::Install => write!(f, "{}", String::from("install")),
            RequestMethod::Remove => write!(f, "{}", String::from("remove")),
        }
    }
}

struct Job {
    method: RequestMethod,
    req: Requirement,
}

impl Job {
    pub fn new(method: RequestMethod, req: Requirement) -> Job {
        Job { method, req }
    }
}

struct Request {
    jobs: Vec<Job>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            jobs: Vec::default(),
        }
    }

    fn install(&mut self, req: Requirement) -> &mut Self {
        self.jobs.push(Job {
            method: RequestMethod::Install,
            req,
        });
        self
    }

    fn remove(&mut self, req: Requirement) -> &mut Self {
        self.jobs.push(Job {
            method: RequestMethod::Remove,
            req,
        });
        self
    }
}

#[derive(Clone)]
struct Package {
    name: String,
    version: Version,
    publisher: String,

    os: OS,
    arch: Arch,
    summary: String,
    description: String,

    requirements: Vec<Box<Requirement>>,

    channels: Vec<Box<String>>,

    location: i32,
    priority: i32,
}

impl Package {
    pub fn new(
        name: String,
        version: Version,
        publisher: String,
        os: OS,
        arch: Arch,
        summary: String,
        description: String,
    ) -> Package {
        Package {
            name,
            version,
            publisher,
            os,
            arch,
            summary,
            description,
            requirements: Vec::default(),
            channels: Vec::default(),
            location: 0,
            priority: 10,
        }
    }
    
    pub fn from(manifest: Manifest) -> Result<Package, Error> {
        Ok(Package {
            name: manifest.zpkg.name,
            version: Version::from(manifest.zpkg.version)?,
            publisher: manifest.zpkg.publisher,
            os: OS::from_str(&manifest.zpkg.os)?,
            arch: Arch::from_str(&manifest.zpkg.arch)?,
            summary: manifest.zpkg.summary,
            description: manifest.zpkg.description,
            requirements: vec![],
            channels: vec![],
            location: 0,
            priority: 10
        })
    }

    fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    fn file_name(&self) -> String {
        format!(
            "{}@{}-{}-{}.zpkg",
            self.name, self.version, self.os, self.arch
        )
    }

    fn satisfies(&self, req: Requirement) -> bool {
        if self.name != req.name {
            return false;
        }

        match req.comparator {
            Comparator::ANY => true,
            Comparator::EXQ => match req.version {
                Some(v) => self.version.exq(&v),
                None => false,
            },
            Comparator::GTE => match req.version {
                Some(v) => self.version >= v,
                None => false,
            },
            Comparator::EQ => match req.version {
                Some(v) => self.version == v,
                None => false,
            },
            Comparator::LTE => match req.version {
                Some(v) => self.version <= v,
                None => false,
            },
        }
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name > other.name {
            return Ordering::Greater;
        }

        if self.name < other.name {
            return Ordering::Less;
        }

        // Lower is higher for priority
        if self.priority > other.priority {
            return Ordering::Less;
        }

        if self.priority < other.priority {
            return Ordering::Greater;
        }

        if self.version > other.version {
            return Ordering::Greater;
        }

        if self.version < other.version {
            return Ordering::Less;
        }

        Ordering::Equal
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Package {}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.version.to_string() == other.version.to_string()
            && self.priority == other.priority
    }
}

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

#[derive(Clone)]
struct Repo {
    uri: Url,

    priority: u32,
    enabled: bool,
    updated: DateTime<Utc>,

    packages: HashSet<Package>,

    channels: HashSet<String>,
}

impl Repo {
    pub fn new(uri: Url, priority: u32, enabled: bool) -> Repo {
        Repo {
            uri,
            priority,
            enabled,
            updated: Utc::now(),
            packages: HashSet::default(),
            channels: HashSet::default(),
        }
    }

    fn add(&mut self, packages: &[Package]) -> Vec<Package> {
        let mut rejects: Vec<Package> = Vec::new();

        for pkg in packages {
            if !self.packages.contains(&pkg) {
                self.packages.insert(pkg.clone());
            } else {
                rejects.push(pkg.clone());
            }
        }

        rejects.sort();
        rejects
    }

    fn remove(&mut self, package: &Package) {
        if self.packages.contains(package) {
            self.packages.remove(package);
        }
    }

    fn contains(self, package: &Package) -> bool {
        self.packages.contains(package)
    }

    fn contents_for_name(&mut self, name: String) -> Vec<Package> {
        self.packages
            .iter()
            .filter(|p| p.name == name)
            .map(|v| v.clone())
            .collect()
    }

    fn prune(&mut self, count: i32) -> Option<Vec<Package>> {
        let mut pruned: Vec<Package> = Vec::new();
        let mut result: Vec<Package> = Vec::new();
        let mut names: Vec<String> = self.packages.iter().map(|p| p.name.clone()).collect();

        names.sort();
        names.dedup();

        for name in names {
            let packages = self.contents_for_name(name);

            let mut current: VecDeque<Package> = VecDeque::from_iter(packages);

            if current.len() > count as usize {
                let offset = current.len() - count as usize;

                for _ in 0..offset {
                    pruned.push(current.pop_front().unwrap());
                }
            }

            result.append(Vec::from(current).as_mut())
        }

        self.packages = HashSet::from_iter(result);

        if pruned.len() == 0 {
            return None;
        }

        Some(pruned)
    }

    fn load(&mut self, packages: Vec<Package>) {
        for pkg in packages {
            self.packages.insert(pkg);
        }
    }

    fn contents(self) -> Vec<Package> {
        let mut filtered: Vec<Package> = Vec::new();

        for pkg in self.packages {
            if self.channels.len() > 0 {
                for ch in &self.channels {
                    if pkg.channels.iter().any(|c| c.as_str() == ch) {
                        filtered.push(pkg.clone())
                    }
                }
            } else {
                filtered.push(pkg.clone())
            }
        }

        filtered.sort();
        filtered
    }
}

impl Ord for Repo {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower is higher for priority
        if self.priority > other.priority {
            return Ordering::Less;
        }

        if self.priority < other.priority {
            return Ordering::Greater;
        }

        if self.uri.to_string() > other.uri.to_string() {
            return Ordering::Greater;
        }

        if self.uri.to_string() < self.uri.to_string() {
            return Ordering::Less;
        }

        Ordering::Equal
    }
}

impl PartialOrd for Repo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Repo {}

impl PartialEq for Repo {
    fn eq(&self, other: &Self) -> bool {
        self.uri.to_string() == other.uri.to_string() && self.priority == other.priority
    }
}

pub enum Phase {
    Install,
    Remove,
    Package,
    Configure,
    NOOP,
    Validate
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
        let v1 = Version::from("3.2.1:20200515T194203Z")?;
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
        assert_eq!(
            req_long.version.unwrap().to_string(),
            "3.2.1:20200415T194203Z"
        );

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

    #[test]
    fn test_request() {
        let mut req = Request::new();

        req.install(Requirement::new(
            String::from("zps"),
            RequirementMethod::Depends,
            Comparator::ANY,
            None,
        ));

        req.remove(Requirement::new(
            String::from("apt"),
            RequirementMethod::Depends,
            Comparator::ANY,
            None,
        ));

        for (index, job) in req.jobs.iter().enumerate() {
            match index {
                0 => {
                    assert_eq!(job.method, RequestMethod::Install);
                    assert_eq!(job.req.name, String::from("zps"));
                }
                1 => {
                    assert_eq!(job.method, RequestMethod::Remove);
                    assert_eq!(job.req.name, String::from("apt"));
                }
                _ => (),
            }
        }
    }

    #[test]
    fn test_package_sort() {
        let mut packages: Vec<Package> = Vec::new();

        let snarf13 = Package::new(
            String::from("snarf"),
            Version::from("1.3.4:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );
        packages.push(snarf13);

        let snarf12 = Package::new(
            String::from("snarf"),
            Version::from("1.2.4:20200515T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );
        packages.push(snarf12);

        let mut snarf_prio = Package::new(
            String::from("snarf"),
            Version::from("1.2.4:20200515T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );
        snarf_prio.priority = 2;
        packages.push(snarf_prio);

        let beef10 = Package::new(
            String::from("beef"),
            Version::from("1.0.0:20200615T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );
        packages.push(beef10);

        let beef11 = Package::new(
            String::from("beef"),
            Version::from("1.0.0:20200515T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );
        packages.push(beef11);

        packages.sort();

        let names: Vec<String> = packages.iter().map(|pkg| pkg.id()).collect();
        assert_eq!(
            names,
            [
                "beef@1.0.0:20200515T194203Z",
                "beef@1.0.0:20200615T194203Z",
                "snarf@1.2.4:20200515T194203Z",
                "snarf@1.3.4:20200415T194203Z",
                "snarf@1.2.4:20200515T194203Z",
            ]
        )
    }

    #[test]
    fn test_package_satisfies() {
        let snarf = Package::new(
            String::from("snarf"),
            Version::from("1.3.4:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarfing pretty hard"),
            String::from("snarfing pretty hard"),
        );

        assert_eq!(
            true,
            snarf.satisfies(Requirement {
                name: "snarf".to_string(),
                method: RequirementMethod::Depends,
                comparator: Comparator::ANY,
                version: None
            })
        );

        assert_eq!(
            false,
            snarf.satisfies(Requirement {
                name: "snarf".to_string(),
                method: RequirementMethod::Depends,
                comparator: Comparator::EXQ,
                version: Some(Version::from("1.3.4:20200515T194203Z").unwrap())
            })
        );

        assert_eq!(
            true,
            snarf.satisfies(Requirement {
                name: "snarf".to_string(),
                method: RequirementMethod::Depends,
                comparator: Comparator::GTE,
                version: Some(Version::from("1.3.2:20200515T194203Z").unwrap())
            })
        );

        assert_eq!(
            true,
            snarf.satisfies(Requirement {
                name: "snarf".to_string(),
                method: RequirementMethod::Depends,
                comparator: Comparator::LTE,
                version: Some(Version::from("1.3.5:20200515T194203Z").unwrap())
            })
        );
    }

    #[test]
    fn test_repo_create() {
        let core = Repo::new(Url::parse("s3://somepath/zps.io/core").unwrap(), 8, true);

        let util = Repo::new(Url::parse("s3://somepath/zps.io/util").unwrap(), 10, true);

        let mut repos: Vec<Repo> = vec![core, util];

        repos.sort();
        repos.reverse();

        assert_eq!(
            "s3://somepath/zps.io/core",
            repos.get(0).unwrap().uri.to_string()
        );
        assert_eq!(
            "s3://somepath/zps.io/util",
            repos.get(1).unwrap().uri.to_string()
        );
    }

    #[test]
    fn test_repo_contents() {
        let mut repo = Repo::new(Url::parse("s3://somepath/zps.io/core").unwrap(), 8, true);

        let zps = Package::new(
            String::from("zps"),
            Version::from("1.3.4:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("zps the last word"),
            String::from("zps the last word"),
        );

        let zps1 = Package::new(
            String::from("zps"),
            Version::from("1.3.5:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("zps the last word"),
            String::from("zps the last word"),
        );

        let snarf = Package::new(
            String::from("snarf"),
            Version::from("1.0.0:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarf the man"),
            String::from("snarf the man"),
        );

        let snarf1 = Package::new(
            String::from("snarf"),
            Version::from("1.0.1:20200415T194203Z").unwrap(),
            String::from("zps.io"),
            OS::Linux,
            Arch::X8664,
            String::from("snarf the man"),
            String::from("snarf the man"),
        );

        repo.load(vec![zps, zps1, snarf, snarf1]);

        let contents = repo.contents();
        assert_eq!(
            "snarf@1.0.0:20200415T194203Z",
            contents.get(0).unwrap().id()
        );
        assert_eq!(
            "snarf@1.0.1:20200415T194203Z",
            contents.get(1).unwrap().id()
        );
        assert_eq!("zps@1.3.4:20200415T194203Z", contents.get(2).unwrap().id());
        assert_eq!("zps@1.3.5:20200415T194203Z", contents.get(3).unwrap().id());
    }
}
