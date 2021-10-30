/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */
use crate::action::*;
use anyhow::*;
use std::collections::HashSet;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct Manifest {
    pub zpkg: Zpkg,
    pub dirs: Vec<Dir>,
    pub files: Vec<File>,
}

// TODO resolve sorting of action vectors
impl Manifest {
    pub fn new(zpkg: Zpkg) -> Self {
        Self {
            zpkg,
            dirs: vec![],
            files: vec![]
        }
    }
    
    pub fn actions(&self) -> Vec<Box<dyn Action>> {
        let mut actions: Vec<Box<dyn Action>> = Vec::new();

        actions.push(Box::new(self.zpkg.clone()));

        for action in self.dirs.iter() {
            actions.push(Box::new(action.clone()));
        }

        for action in self.files.iter() {
            actions.push(Box::new(action.clone()));
        }

        actions
    }

    pub fn add(&mut self, action: Box<dyn Action>) {
        match action.type_name() {
            ActionType::Zpkg => {
                self.zpkg = action.as_any().downcast_ref::<Zpkg>().unwrap().clone();
            },
            ActionType::Dir => {
                if !self.dirs.contains(action.as_any().downcast_ref::<Dir>().unwrap()) {
                    self.dirs.push(action.as_any().downcast_ref::<Dir>().unwrap().clone());
                }
            },
            ActionType::File => {
                if !self.files.contains(action.as_any().downcast_ref::<File>().unwrap()) {
                    self.files.push(action.as_any().downcast_ref::<File>().unwrap().clone());
                }
            }
        }
    }

    pub fn set(&mut self, actions: Vec<Box<dyn Action>>) {
        self.dirs = Vec::new();
        self.files = Vec::new();

        for action in actions {
            match action.type_name() {
                ActionType::Zpkg => {
                  self.zpkg = action.as_any().downcast_ref::<Zpkg>().unwrap().clone();
                },
                ActionType::Dir => {
                    self.dirs.push(action.as_any().downcast_ref::<Dir>().unwrap().clone());
                },
                ActionType::File => {
                    self.files.push(action.as_any().downcast_ref::<File>().unwrap().clone());
                }
            }
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>, Error> {
        match serde_json::ser::to_vec(self) {
            Ok(result) => Ok(result),
            Err(err) => Err(Error::from(err))
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        // Ensure integrity of FS objects
        let mut index : HashSet<String> = HashSet::new();

        for action in self.dirs.iter() {
            if index.contains(action.key().as_str()) {
                return Err(anyhow!("duplicate action for key: {}", action.key()))
            }
            index.insert(action.key());
        }

        for action in self.files.iter() {
            if index.contains(action.key().as_str()) {
                return Err(anyhow!("duplicate action for key: {}", action.key()))
            }
            index.insert(action.key());
        }

        Ok(())
    }
}