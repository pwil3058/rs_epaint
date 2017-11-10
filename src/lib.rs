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

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate pw_gix;

extern crate regex;

extern crate cairo;
extern crate gdk;
extern crate gtk;

pub mod characteristics;
pub mod colour_mix;
pub mod components;
pub mod hue_wheel;
pub mod mixed;
pub mod mixer;
pub mod model_paint;
pub mod paint;
pub mod series;
pub mod shape;
pub mod target;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
