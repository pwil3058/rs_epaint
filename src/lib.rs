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
    use std::convert::From;
    use std::error::Error;
    use std::fmt;
    use std::io;

    use regex;

    use pw_gix::rgb_math::rgb;

    #[derive(Debug)]
    pub enum PaintErrorType {
        AlreadyExists(String),
        MalformedText(String),
        IOError(io::Error),
        NoSubstantiveComponents,
        NoCollectionId,
        UserCancelled,
    }

    #[derive(Debug)]
    pub struct PaintError {
        error_type: PaintErrorType,
        msg: String,
    }

    impl PaintError {
        pub fn error_type(&self) -> &PaintErrorType {
            &self.error_type
        }
    }

    impl From<PaintErrorType> for PaintError {
        fn from(error_type: PaintErrorType) -> PaintError {
            let msg = match error_type {
                PaintErrorType::AlreadyExists(ref text) => format!("{}: already exists.", text),
                PaintErrorType::MalformedText(ref text) => format!("{}: is (or contains) malformed text.", text),
                PaintErrorType::IOError(ref io_error) => format!("I/O Error: {}", io_error.description()),
                PaintErrorType::NoSubstantiveComponents => "Contains no nonzero components.".to_string(),
                PaintErrorType::NoCollectionId => "Missing collection identifier.".to_string(),
                PaintErrorType::UserCancelled => "Operation cancelled by the user.".to_string(),
            };
            PaintError{error_type, msg}
        }
    }

    impl From<io::Error> for PaintError {
        fn from(io_error: io::Error) -> PaintError {
            PaintErrorType::IOError(io_error).into()
        }
    }

    impl From<regex::Error> for PaintError {
        fn from(regex_error: regex::Error) -> PaintError {
            match regex_error {
                regex::Error::Syntax(string) => PaintErrorType::MalformedText(string).into(),
                _ => panic!("Unexpected regex error"),
            }

        }
    }

    impl From<rgb::RGBError> for PaintError {
        fn from(rgb_error: rgb::RGBError) -> PaintError {
            match rgb_error {
                rgb::RGBError::MalformedText(string) => PaintErrorType::MalformedText(string).into(),
            }

        }
    }

    impl fmt::Display for PaintError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "PaintError({:?}): {}.", self.error_type, self.msg)?;
            Ok(())
        }
    }

    impl Error for PaintError {
        fn description(&self) -> &str {
            &self.msg
        }
    }

    pub type PaintResult<T> = Result<T, PaintError>;
}

pub mod display {
    pub struct PaintDisplayButtonSpec {
        pub label: String,
        pub tooltip_text: String,
        pub callback: Box<Fn()>
    }
}

pub mod basic_paint;
pub mod characteristics;
pub mod colln_paint;
pub mod colour_edit;
pub mod colour_mix;
pub mod icons;
pub mod graticule;
pub mod mixed_paint;
pub mod model_paint;
pub mod series_paint;
pub mod shape;

use std::env;

pub fn app_name() -> String {
    if let Some(ref text) = env::args().next() {
        pw_pathux::split_path_text(text).1.to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
