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

    use basic_paint::CharacteristicsInterface;
    use characteristics::CharacteristicError;
    use mixed_paint::MixedPaint;

    #[derive(Debug)]
    pub enum PaintErrorType<C: CharacteristicsInterface> {
        AlreadyExists(String),
        MalformedText(String),
        NotFound(String),
        IOError(io::Error),
        NoSubstantiveComponents,
        NoCollectionId,
        UserCancelled,
        BeingUsedBy(Vec<MixedPaint<C>>),
        PartOfCurrentMixture,
    }

    #[derive(Debug)]
    pub struct PaintError<C: CharacteristicsInterface> {
        error_type: PaintErrorType<C>,
        msg: String,
    }

    impl<C: CharacteristicsInterface> PaintError<C> {
        pub fn error_type(&self) -> &PaintErrorType<C> {
            &self.error_type
        }
    }

    impl<C: CharacteristicsInterface> From<PaintErrorType<C>> for PaintError<C> {
        fn from(error_type: PaintErrorType<C>) -> PaintError<C> {
            let msg = match error_type {
                PaintErrorType::AlreadyExists(ref text) => format!("{}: already exists.", text),
                PaintErrorType::MalformedText(ref text) => format!("{}: is (or contains) malformed text.", text),
                PaintErrorType::NotFound(ref text) => format!("{}: not found.", text),
                PaintErrorType::IOError(ref io_error) => format!("I/O Error: {}", io_error.description()),
                PaintErrorType::NoSubstantiveComponents => "Contains no nonzero components.".to_string(),
                PaintErrorType::NoCollectionId => "Missing collection identifier.".to_string(),
                PaintErrorType::UserCancelled => "Operation cancelled by the user.".to_string(),
                PaintErrorType::BeingUsedBy(_) => "Is being used as a component by one or more paints.".to_string(),
                PaintErrorType::PartOfCurrentMixture => "Is being used as a component of the current mixture.".to_string(),
            };
            PaintError{error_type, msg}
        }
    }

    impl<C: CharacteristicsInterface> From<CharacteristicError> for PaintError<C> {
        fn from(ch_error: CharacteristicError) -> PaintError<C> {
            let text = ch_error.description().to_string();
            PaintError{error_type: PaintErrorType::MalformedText(text.clone()), msg: text}
        }
    }

    impl<C: CharacteristicsInterface> From<io::Error> for PaintError<C> {
        fn from(io_error: io::Error) -> PaintError<C> {
            PaintErrorType::IOError(io_error).into()
        }
    }

    impl<C: CharacteristicsInterface> From<regex::Error> for PaintError<C> {
        fn from(regex_error: regex::Error) -> PaintError<C> {
            match regex_error {
                regex::Error::Syntax(string) => PaintErrorType::MalformedText(string).into(),
                _ => panic!("Unexpected regex error"),
            }

        }
    }

    impl<C: CharacteristicsInterface> From<rgb::RGBError> for PaintError<C> {
        fn from(rgb_error: rgb::RGBError) -> PaintError<C> {
            match rgb_error {
                rgb::RGBError::MalformedText(string) => PaintErrorType::MalformedText(string).into(),
            }

        }
    }

    impl<C: CharacteristicsInterface> fmt::Display for PaintError<C> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "PaintError({:?}): {}.", self.error_type, self.msg)?;
            Ok(())
        }
    }

    impl<C: CharacteristicsInterface> Error for PaintError<C> {
        fn description(&self) -> &str {
            &self.msg
        }
    }

    pub type PaintResult<T, C> = Result<T, PaintError<C>>;
}

pub mod dialogue {
    use std::cell::RefCell;
    use std::rc::Rc;

    use glib::signal::SignalHandlerId;
    use gtk;
    use gtk::{DialogExt, GtkWindowExt, WidgetExt};

    use pw_gix::colour::*;
    use pw_gix::wrapper::{parent_none, WidgetWrapper};

    use super::app_name;
    use super::basic_paint::{BasicPaintInterface, CharacteristicsInterface, ColourAttributesInterface};

    pub struct PaintDisplayButtonSpec {
        pub label: String,
        pub tooltip_text: String,
        pub callback: Box<Fn()>
    }

    pub fn new_display_dialog<W>(title: &str, caller: &Rc<W>, buttons: &[(&str, i32)]) -> gtk::Dialog
        where   W: WidgetWrapper
    {
        let title = format!("{}: {}", app_name(), title);
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent_none(),
            gtk::DialogFlags::USE_HEADER_BAR,
            buttons
        );
        if let Some(tlw) = caller.get_toplevel_gtk_window() {
            dialog.set_transient_for(Some(&tlw));
            if let Some(ref icon) = tlw.get_icon() {
                dialog.set_icon(Some(icon));
            }
        };
        dialog
    }

    pub trait DialogWrapper {
        fn dialog(&self) -> gtk::Dialog;

        fn show(&self) {
            self.dialog().show()
        }

        fn present(&self) {
            self.dialog().present()
        }

        fn close(&self) {
            self.dialog().close()
        }

        fn set_response_sensitive(&self, response_id: i32, setting: bool) {
            self.dialog().set_response_sensitive(response_id, setting);
        }

        fn connect_close<F: Fn(&gtk::Dialog) + 'static>(&self, f: F) -> SignalHandlerId {
            self.dialog().connect_close(f)
        }

        fn connect_destroy<F: Fn(&gtk::Dialog) + 'static>(&self, f: F) -> SignalHandlerId {
            self.dialog().connect_destroy(f)
        }
    }

    static mut NEXT_DIALOG_ID: u32 = 0;

    pub fn get_id_for_dialog() -> u32 {
        let id: u32;
        unsafe {
            id = NEXT_DIALOG_ID;
            NEXT_DIALOG_ID += 1;
        }
        id
    }

    pub type DestroyedCallbacks = RefCell<Vec<Box<Fn(u32)>>>;

    pub trait DestroyedCallbacksIfce {
        fn create() -> DestroyedCallbacks;
    }

    impl DestroyedCallbacksIfce for DestroyedCallbacks {
        fn create() -> DestroyedCallbacks { RefCell::new(Vec::new()) }
    }

    pub trait TrackedDialog {
        fn id_no(&self) -> u32;
        fn destroyed_callbacks(&self) -> &DestroyedCallbacks;

        fn connect_destroyed<F: 'static + Fn(u32)>(&self, callback: F) {
            self.destroyed_callbacks().borrow_mut().push(Box::new(callback))
        }

        fn inform_destroyed(&self) {
            for callback in self.destroyed_callbacks().borrow().iter() {
                callback(self.id_no());
            }
        }
    }

    pub trait PaintDisplay<A, C, P>: DialogWrapper + TrackedDialog
        where   C: CharacteristicsInterface + 'static,
                A: ColourAttributesInterface + 'static,
                P: BasicPaintInterface<C> + 'static,
    {
        fn create<W: WidgetWrapper>(
            paint: &P,
            caller: &Rc<W>,
            button_specs: Vec<PaintDisplayButtonSpec>,
        ) -> Self;

        fn paint(&self) -> P;
    }

    pub trait PaintDisplayWithCurrentTarget<A, C, P>: DialogWrapper + TrackedDialog
        where   C: CharacteristicsInterface + 'static,
                A: ColourAttributesInterface + 'static,
                P: BasicPaintInterface<C> + 'static,
    {
        fn create<W: WidgetWrapper>(
            paint: &P,
            current_target: Option<&Colour>,
            caller: &Rc<W>,
            button_specs: Vec<PaintDisplayButtonSpec>,
        ) -> Self;

        fn paint(&self) -> P;
        fn set_current_target(&self, new_current_target: Option<&Colour>);
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
pub mod standards;

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
