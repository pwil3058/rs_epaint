// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::*;
use std::rc::Rc;

use colour::*;

pub struct TargetColourCore {
    name: String,
    notes: String,
    colour: Colour,
}

impl TargetColourCore {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn notes(&self) -> String {
        self.notes.clone()
    }

    pub fn tooltip_text(&self) -> String {
        format!("{}: {}", self.name, self.notes)
    }

    pub fn colour(&self) -> Colour {
        self.colour.clone()
    }
}

impl PartialEq for TargetColourCore {
    fn eq(&self, other: &TargetColourCore) -> bool {
        self.name == other.name
    }
}

impl Eq for TargetColourCore {}

impl PartialOrd for TargetColourCore {
    fn partial_cmp(&self, other: &TargetColourCore) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for TargetColourCore {
    fn cmp(&self, other: &TargetColourCore) -> Ordering {
        self.name.cmp(&other.name)
    }
}

pub type TargetColour = Rc<TargetColourCore>;

pub trait TargetColourInterface {
    fn create(colour: &Colour, name: &str, notes: &str) -> TargetColour;
}

impl TargetColourInterface for TargetColour {
    fn create(colour: &Colour, name: &str, notes: &str) -> TargetColour {
        Rc::new(
            TargetColourCore{
                colour: colour.clone(),
                name: name.to_string(),
                notes: notes.to_string(),
            }
        )
    }
}


#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
