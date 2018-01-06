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

extern crate pw_pathux;

extern crate chrono;
extern crate num;
extern crate regex;
extern crate xml;

extern crate cairo;
extern crate gdk;
extern crate glib;
extern crate gtk;
extern crate gdk_pixbuf;

pub mod struct_traits {
    pub trait SimpleCreation {
        fn create() -> Self;
    }

    pub trait SingleArgCreation<A> {
        fn create(a: &A) -> Self;
    }

    pub trait DoubleArgCreation<A, B> {
        fn create(a: &A, b: &B) -> Self;
    }

    pub trait TripleArgCreation<A, B, C> {
        fn create(a: &A, b: &B, c: &C) -> Self;
    }
}

pub mod error {
    use std::io;

    #[derive(Debug)]
    pub enum PaintError {
        AlreadyExists(String),
        MalformedText(String),
        PaintTypeMismatch,
        IOError(io::Error),
        NoSubstantiveComponents,
        NotFound(String),
        DifferentPaintSameName
    }
}

pub mod basic_paint;
pub mod characteristics;
pub mod colln_paint;
pub mod colour_edit;
pub mod colour_mix;
pub mod display;
pub mod hue_wheel;
pub mod icons;
pub mod graticule;
pub mod mixed_paint;
pub mod model_paint;
pub mod paint;
pub mod series_paint;
pub mod shape;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
