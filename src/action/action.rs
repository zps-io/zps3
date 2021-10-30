/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2020 Zachary Schneider
 */

use std::any::Any;

use strum_macros::{EnumIter, EnumString, ToString};

#[derive(Copy, Clone, Debug, PartialEq, EnumIter, EnumString, ToString)]
pub enum ActionType {
    Dir,
    File,
    Zpkg
}

pub trait Action {
    fn id(&self) -> String;
    fn key(&self) -> String;
    fn type_name(&self) -> ActionType;

    fn is_valid(&self) -> bool;
    fn to_string(&self) -> String;

    fn as_any(&self) -> &dyn Any;
}

impl<'a> PartialEq for dyn Action + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
