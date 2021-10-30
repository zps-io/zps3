/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use super::action::{Action, ActionType};
use std::any::Any;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct Zpkg {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub arch: String,
    pub os: String,
    pub summary: String,
    pub description: String,
}

impl Action for Zpkg {
    fn id(&self) -> String {
        format!("{}:{}", self.type_name().to_string(), self.name)
    }

    fn key(&self) -> String {
        self.name.clone()
    }

    fn type_name(&self) -> ActionType  {
        ActionType::Zpkg
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() &&
            !self.version.is_empty() &&
            !self.publisher.is_empty() &&
            !self.arch.is_empty() &&
            !self.os.is_empty() &&
            !self.summary.is_empty() &&
            !self.description.is_empty()
    }

    fn to_string(&self) -> String {
        format!("{} {}@{}", self.type_name().to_string(), self.name, self.version)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}