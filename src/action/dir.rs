/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use std::any::Any;

use super::action::Action;
use crate::action::action::ActionType;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Dir {
    pub path: String,
    pub owner: String,
    pub group: String,
    pub mode: u32,
}

impl Action for Dir {
    fn id(&self) -> String {
        format!("{}:{}", self.type_name().to_string(), self.path)
    }

    fn key(&self) -> String {
        self.path.clone()
    }

    fn type_name(&self) -> ActionType {
        ActionType::Dir
    }

    fn is_valid(&self) -> bool {
        !self.path.is_empty()
    }

    fn to_string(&self) -> String {
        format!("{} {}:{} {:o} {}", self.type_name().to_string(), self.owner, self.group, self.mode, self.path)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for Dir {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}