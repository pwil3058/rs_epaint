// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[macro_use]
extern crate lazy_static;

use pw_pathux;

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

pub mod colour {
    use std::cmp::Ordering;

    use serde_derive::*;

    use normalised_angles::Degrees;

    use colour_math::{RGBA, HCV};
    pub use colour_math::{
        urgb::{URGBError, RGB16, RGB8},
        ColourInterface, HueConstants, RGBConstants, ScalarAttribute,
    };
    use pw_gix::gdk;

    pub type HCV = colour_math::hcv::HCV<f64>;
    pub type Hue = colour_math::hue::Hue<f64>;
    pub type HueData = colour_math::chroma::HueData<f64>;
    pub type RGB = colour_math::rgb::RGB<f64>;
    pub type RGBManipulator = colour_math::manipulator::ColourManipulator<f64>;
    pub type ColourManipulatorBuilder = colour_math::manipulator::ColourManipulatorBuilder<f64>;

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    pub struct Colour {
        rgb: RGB,
        hcv: HCV,
    }

    impl PartialEq for Colour {
        fn eq(&self, other: &Self) -> bool {
            self.rgb == other.rgb
        }
    }

    impl Eq for Colour {}

    impl PartialOrd for Colour {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            if self.rgb == other.rgb {
                Some(Ordering::Equal)
            } else if let Some(hue) = self.hue {
                if let Some(other_hue) = other.hue {
                    // This orders via hue from CYAN to CYAN via GREEN, RED, BLUE in that order
                    match hue.partial_cmp(&other_hue) {
                        Some(Ordering::Less) => Some(Ordering::Less),
                        Some(Ordering::Greater) => Some(Ordering::Greater),
                        Some(Ordering::Equal) => self.rgb.value().partial_cmp(&other.rgb.value()),
                        None => None,
                    }
                } else {
                    Some(Ordering::Greater)
                }
            } else if other.hue.is_some() {
                Some(Ordering::Less)
            } else {
                self.rgb.value().partial_cmp(&other.rgb.value())
            }
        }
    }

    impl From<RGB> for Colour {
        fn from(rgb: RGB) -> Self {
            use std::convert::TryInto;
            let hcv: HCV = rgb.into();
            Self { rgb, hcv }
        }
    }

    impl ColourInterface<f64> for Colour {
        fn rgb(&self) -> RGB {
            self.rgb
        }

        fn rgba(&self) -> RGBA<f64> {
            self.rgb.rgba()
        }

        fn hcv(&self) -> HCV {
            *self.rgb.hcv
        }

        fn hue(&self) -> Option<Hue> {
            self.hue
        }

        fn hue_angle(&self) -> Option<Degrees<f64>> {
            self.hcv.hue_angle()
        }

        fn is_grey(&self) -> bool {
            self.hcv.is_grey()
        }

        fn chroma(&self) -> f64 {
            self.hcv.chroma()
        }

        fn greyness(&self) -> f64 {
            self.hcv.greyness()
        }

        fn value(&self) -> f64 {
            self.hcv.value()
        }

        fn max_chroma_rgb(&self) -> RGB {
            self.hcv.max_chroma_rgb()
        }

        fn warmth(&self) -> f64 {
            self.rgb.warmth()
        }

        fn best_foreground_rgb(&self) -> RGB {
            self.rgb.best_foreground_rgb()
        }

        fn monochrome_rgb(&self) -> RGB {
            self.hcv.monochrome_rgb()
        }

        fn warmth_rgb(&self) -> RGB {
            self.rgb.warmth_rgb()
        }
    }

    pub trait GdkConvert {
        fn into_gdk_rgba(&self) -> gdk::RGBA;
    }

    impl GdkConvert for RGB {
        fn into_gdk_rgba(&self) -> gdk::RGBA {
            gdk::RGBA {
                red: self[0],
                green: self[1],
                blue: self[2],
                alpha: 1.0,
            }
        }
    }
}

pub mod error {
    use std::convert::From;
    use std::error::Error;
    use std::fmt;
    use std::io;

    use regex;

    use crate::basic_paint::CharacteristicsInterface;
    use crate::characteristics::CharacteristicError;
    use crate::colour::*;
    use crate::mixed_paint::MixedPaint;

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
                PaintErrorType::MalformedText(ref text) => {
                    format!("{}: is (or contains) malformed text.", text)
                }
                PaintErrorType::NotFound(ref text) => format!("{}: not found.", text),
                PaintErrorType::IOError(ref io_error) => {
                    format!("I/O Error: {}", io_error.to_string())
                }
                PaintErrorType::NoSubstantiveComponents => {
                    "Contains no nonzero components.".to_string()
                }
                PaintErrorType::NoCollectionId => "Missing collection identifier.".to_string(),
                PaintErrorType::UserCancelled => "Operation cancelled by the user.".to_string(),
                PaintErrorType::BeingUsedBy(_) => {
                    "Is being used as a component by one or more paints.".to_string()
                }
                PaintErrorType::PartOfCurrentMixture => {
                    "Is being used as a component of the current mixture.".to_string()
                }
            };
            PaintError { error_type, msg }
        }
    }

    impl<C: CharacteristicsInterface> From<CharacteristicError> for PaintError<C> {
        fn from(ch_error: CharacteristicError) -> PaintError<C> {
            let text = ch_error.to_string();
            PaintError {
                error_type: PaintErrorType::MalformedText(text.clone()),
                msg: text,
            }
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

    impl<C: CharacteristicsInterface> From<URGBError> for PaintError<C> {
        fn from(rgb_error: URGBError) -> PaintError<C> {
            match rgb_error {
                URGBError::MalformedText(string) => PaintErrorType::MalformedText(string).into(),
            }
        }
    }

    impl<C: CharacteristicsInterface> fmt::Display for PaintError<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    use pw_gix::{
        glib::signal::SignalHandlerId,
        gtk::{self, prelude::GtkWindowExtManual, DialogExt, GtkWindowExt, WidgetExt},
        wrapper::{parent_none, WidgetWrapper},
    };

    use super::app_name;
    use super::basic_paint::{
        BasicPaintInterface, CharacteristicsInterface, ColourAttributesInterface,
    };
    use super::colour::*;

    pub struct PaintDisplayButtonSpec {
        pub label: String,
        pub tooltip_text: String,
        pub callback: Box<dyn Fn()>,
    }

    pub fn new_display_dialog<W>(
        title: &str,
        caller: &Rc<W>,
        buttons: &[(&str, gtk::ResponseType)],
    ) -> gtk::Dialog
    where
        W: WidgetWrapper,
    {
        let title = format!("{}: {}", app_name(), title);
        let dialog = gtk::Dialog::with_buttons(
            Some(title.as_str()),
            parent_none(),
            gtk::DialogFlags::USE_HEADER_BAR,
            buttons,
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

        fn set_response_sensitive(&self, response_id: gtk::ResponseType, setting: bool) {
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

    pub type DestroyedCallbacks = RefCell<Vec<Box<dyn Fn(u32)>>>;

    pub trait DestroyedCallbacksIfce {
        fn create() -> DestroyedCallbacks;
    }

    impl DestroyedCallbacksIfce for DestroyedCallbacks {
        fn create() -> DestroyedCallbacks {
            RefCell::new(Vec::new())
        }
    }

    pub trait TrackedDialog {
        fn id_no(&self) -> u32;
        fn destroyed_callbacks(&self) -> &DestroyedCallbacks;

        fn connect_destroyed<F: 'static + Fn(u32)>(&self, callback: F) {
            self.destroyed_callbacks()
                .borrow_mut()
                .push(Box::new(callback))
        }

        fn inform_destroyed(&self) {
            for callback in self.destroyed_callbacks().borrow().iter() {
                callback(self.id_no());
            }
        }
    }

    pub trait PaintDisplay<A, C, P>: DialogWrapper + TrackedDialog
    where
        C: CharacteristicsInterface + 'static,
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
    where
        C: CharacteristicsInterface + 'static,
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

pub mod art_paint;
pub mod basic_paint;
pub mod cairox;
pub mod characteristics;
pub mod colln_paint;
pub mod colour_edit;
pub mod colour_mix;
pub mod graticule;
pub mod icons;
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
