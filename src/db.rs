/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use super::action::*;
use kv::*;

struct State {
    path: String,
    store: Option<Store>,
}

impl State {
    pub fn new(path: &str) -> State {
        State {
            path: path.to_string(),
            store: None,
        }
    }

    fn open(&mut self) -> Result<(), Error> {
        self.store = Some(Store::new(Config::new(self.path.clone()))?);
        Ok(())
    }

    fn packages(&mut self) -> Result<Bucket<String, Json<Manifest>>, Error> {
        if self.store.is_none() {
            self.open()?;
        }

        self
            .store
            .as_ref()
            .unwrap()
            .bucket::<String, Json<Manifest>>(Some("packages"))
    }

    pub fn pkg_put(&mut self, pkg: Manifest) -> Result<(), Error> {
        if self.store.is_none() {
            self.open()?;
        }

        let packages = self.packages()?;

        packages.set(pkg.zpkg.name.clone(), Json(pkg))?;

        Ok(())
    }

    pub fn pkg_del(&mut self, pkg: String) -> Result<(), Error> {
        let packages = self.packages()?;

        packages.remove(pkg)?;

        Ok(())
    }

    pub fn pkg_list(&mut self) -> Result<Vec<Manifest>, Error> {
        let packages = self.packages()?;
        let mut list: Vec<Manifest> = vec![];

        //packages.iter().map(|i| i?.value().unwrap()).collect()
        for package in packages.iter() {
            list.push(package.unwrap().value::<Json<Manifest>>()?.into_inner());
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{Arch, OS};

    #[test]
    fn test_put_pkg() -> Result<(), Error> {
        let mut state = State::new("/tmp/zpstestput");
        state.pkg_put(Manifest::new( Zpkg {
                name: "test".to_string(),
                version: "1.0.0:20200320T221640Z".to_string(),
                publisher: "fezz.io".to_string(),
                arch: Arch::X8664.to_string(),
                os: OS::Darwin.to_string(),
                summary: "Test zpkg".to_string(),
                description: "Test zpkg, for well testing".to_string()
        }))
    }

    #[test]
    fn test_del_pkg() -> Result<(), Error> {
        let mut state = State::new("/tmp/zpstestdel");

        state.pkg_put(Manifest::new( Zpkg {
            name: "test".to_string(),
            version: "1.0.0:20200320T221640Z".to_string(),
            publisher: "fezz.io".to_string(),
            arch: Arch::X8664.to_string(),
            os: OS::Darwin.to_string(),
            summary: "Test zpkg".to_string(),
            description: "Test zpkg, for well testing".to_string()
        }))?;
        state.pkg_del( "test".to_string())
    }

    #[test]
    fn test_list_pkg() -> Result<(), Error> {
        let mut state = State::new("/tmp/zpstestlist");

        state.pkg_put(Manifest::new( Zpkg {
            name: "test".to_string(),
            version: "1.0.0:20200320T221640Z".to_string(),
            publisher: "fezz.io".to_string(),
            arch: Arch::X8664.to_string(),
            os: OS::Darwin.to_string(),
            summary: "Test zpkg".to_string(),
            description: "Test zpkg, for well testing".to_string()
        }))?;

        state.pkg_put(Manifest::new( Zpkg {
            name: "nacho".to_string(),
            version: "1.1.1:20200320T221640Z".to_string(),
            publisher: "fezz.io".to_string(),
            arch: Arch::X8664.to_string(),
            os: OS::Darwin.to_string(),
            summary: "Nacho fetcher".to_string(),
            description: "Nacho fetches you nachos".to_string()
        }))?;

        state.pkg_put(Manifest::new( Zpkg {
            name: "hodor".to_string(),
            version: "1.1.0:20200320T221640Z".to_string(),
            publisher: "fezz.io".to_string(),
            arch: Arch::X8664.to_string(),
            os: OS::Darwin.to_string(),
            summary: "Hodor util".to_string(),
            description: "Hodor is the ultimate util".to_string()
        }))?;

        for pkg in state.pkg_list()? {
            println!("{}", pkg.zpkg.to_string())
        }

        Ok(())
    }
}
