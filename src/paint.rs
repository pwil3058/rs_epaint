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

use gdk;
use gtk;
use gtk::StaticType;
use gtk::prelude::*;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::*;
use std::io;
use std::rc::Rc;
use std::str::FromStr;

use pw_gix::colour::*;
use pw_gix::rgb_math::rgb::*;
use pw_gix::rgb_math::hue::*;

use mixed_paint::*;
use series_paint::*;

pub trait CharacteristicsInterface:
    Debug + Hash + PartialEq + Clone + Copy + FromStr
{
    type Entry: CharacteristicsEntryInterface<Self>;

    fn tv_row_len() -> usize;
    fn tv_columns(start_col_id: i32) -> Vec<gtk::TreeViewColumn>;
    fn from_floats(floats: &Vec<f64>) -> Self;

    fn tv_rows(&self) -> Vec<gtk::Value>;
    fn gui_display_widget(&self) -> gtk::Box;
    fn to_floats(&self) -> Vec<f64>;
}

pub trait CharacteristicsEntryInterface<C: CharacteristicsInterface> {
    fn create() -> Self;
    fn pwo(&self) -> gtk::Grid;
    fn get_characteristics(&self) -> Option<C>;
    fn set_characteristics(&self, o_characteristics: Option<&C>);
    fn connect_changed<F: 'static + Fn()>(&self, callback: F);
}

pub trait ColourAttributesInterface {
    fn create() -> Rc<Self>;
    fn tv_columns() -> Vec<gtk::TreeViewColumn>;
    fn scalar_attributes() -> Vec<ScalarAttribute>;

    fn pwo(&self) -> gtk::Box;
    fn set_colour(&self, colour: Option<&Colour>);
    fn set_target_colour(&self, target_colour: Option<&Colour>);
}

pub trait ColouredItemInterface {
    fn colour(&self) -> Colour;

    fn rgb(&self) -> RGB {
        self.colour().rgb()
    }

    fn hue(&self) -> HueAngle {
        self.colour().hue()
    }

    fn is_grey(&self) -> bool {
        self.colour().is_grey()
    }

    fn chroma(&self) -> f64 {
        self.colour().chroma()
    }

    fn greyness(&self) -> f64 {
        self.colour().greyness()
    }

    fn value(&self) -> f64 {
        self.colour().value()
    }

    fn warmth(&self) -> f64 {
        self.colour().warmth()
    }

    fn monotone_rgb(&self) -> RGB {
        self.colour().monotone_rgb()
    }

    fn best_foreground_rgb(&self) -> RGB {
        self.colour().best_foreground_rgb()
    }

    fn max_chroma_rgb(&self) -> RGB {
        self.colour().max_chroma_rgb()
    }

    fn warmth_rgb(&self) -> RGB {
        self.colour().warmth_rgb()
    }

    fn scalar_attribute(&self, attr: ScalarAttribute) -> f64 {
        self.colour().scalar_attribute(attr)
    }
}

pub trait BasicPaintInterface<C>: Hash + Clone + PartialEq + ColouredItemInterface
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String;
    fn notes(&self) -> String;
    fn tooltip_text(&self) -> String;
    fn characteristics(&self) -> C;
}

lazy_static! {
    pub static ref STANDARD_PAINT_ROW_SPEC: [gtk::Type; 18] =
        [
            gtk::Type::String,          // 0 Name
            gtk::Type::String,          // 1 Notes
            gtk::Type::String,          // 2 Chroma
            gtk::Type::String,          // 3 Greyness
            gtk::Type::String,          // 4 Value
            gtk::Type::String,          // 5 Warmth
            gdk::RGBA::static_type(),   // 6 RGB
            gdk::RGBA::static_type(),   // 7 FG for RGB
            gdk::RGBA::static_type(),   // 8 Monochrome RGB
            gdk::RGBA::static_type(),   // 9 FG for Monochrome RGB
            gdk::RGBA::static_type(),   // 10 Warmth RGB
            gdk::RGBA::static_type(),   // 11 FG for Warmth RGB
            gdk::RGBA::static_type(),   // 12 Hue Colour
            f64::static_type(),         // 13 Hue angle (radians)
            gtk::Type::String,          // 14 Characteristic #1
            gtk::Type::String,          // 15 Characteristic #2
            gtk::Type::String,          // 16 Characteristic #3
            gtk::Type::String,          // 17 Characteristic #4
        ];
}

pub trait PaintTreeViewColumnData<C>: BasicPaintInterface<C>
    where   C: CharacteristicsInterface
{
    fn tv_row_len() -> usize {
        14 + C::tv_row_len()
    }

    fn tv_rows(&self) -> Vec<gtk::Value> {
        let rgba: gdk::RGBA = self.rgb().into();
        let frgba: gdk::RGBA = self.rgb().best_foreground_rgb().into();
        let mrgba: gdk::RGBA = self.monotone_rgb().into();
        let mfrgba: gdk::RGBA = self.monotone_rgb().best_foreground_rgb().into();
        let wrgba: gdk::RGBA = self.warmth_rgb().into();
        let wfrgba: gdk::RGBA = self.warmth_rgb().best_foreground_rgb().into();
        let hrgba: gdk::RGBA = self.max_chroma_rgb().into();
        let mut rows = vec![
            self.name().to_value(),
            self.notes().to_value(),
            format!("{:5.4}", self.chroma()).to_value(),
            format!("{:5.4}", self.greyness()).to_value(),
            format!("{:5.4}", self.value()).to_value(),
            format!("{:5.4}", self.warmth()).to_value(),
            rgba.to_value(),
            frgba.to_value(),
            mrgba.to_value(),
            mfrgba.to_value(),
            wrgba.to_value(),
            wfrgba.to_value(),
            hrgba.to_value(),
            self.hue().angle().radians().to_value(),
        ];
        for row in self.characteristics().tv_rows().iter() {
            rows.push(row.clone());
        };
        rows
    }
}

pub trait PaintTreeViewColumnSpec {
    fn tv_columns() -> Vec<gtk::TreeViewColumn>;
}

#[derive(Debug, Clone, Hash)]
pub enum Paint<C: CharacteristicsInterface> {
    Series(SeriesPaint<C>),
    Mixed(MixedPaint<C>)
}

impl<C: CharacteristicsInterface> PaintTreeViewColumnData<C> for Paint<C> {}

impl<C: CharacteristicsInterface> Paint<C> {
    pub fn is_series(&self) ->bool {
        match *self {
            Paint::Series(_) => true,
            Paint::Mixed(_) => false
        }

    }

    pub fn is_mixed(&self) ->bool {
        !self.is_series()
    }
}

impl<C: CharacteristicsInterface> PartialEq for Paint<C> {
    fn eq(&self, other: &Paint<C>) -> bool {
        match *self {
            Paint::Series(ref paint) => {
                match *other {
                    Paint::Series(ref opaint) => paint == opaint,
                    Paint::Mixed(_) => false,
                }
            },
            Paint::Mixed(ref paint) => {
                match *other {
                    Paint::Series(_) => false,
                    Paint::Mixed(ref opaint) => paint == opaint,
                }

            }
        }
    }
}

impl<C: CharacteristicsInterface> Eq for Paint<C> {}

impl<C: CharacteristicsInterface> PartialOrd for Paint<C> {
    fn partial_cmp(&self, other: &Paint<C>) -> Option<Ordering> {
        match *self {
            Paint::Series(ref paint) => {
                match *other {
                    Paint::Series(ref opaint) => paint.partial_cmp(opaint),
                    Paint::Mixed(_) => Some(Ordering::Less),
                }
            },
            Paint::Mixed(ref paint) => {
                match *other {
                    Paint::Series(_) => Some(Ordering::Greater),
                    Paint::Mixed(ref opaint) => paint.partial_cmp(opaint),
                }

            }
        }
    }
}

impl<C: CharacteristicsInterface> Ord for Paint<C> {
    fn cmp(&self, other: &Paint<C>) -> Ordering {
        match *self {
            Paint::Series(ref paint) => {
                match *other {
                    Paint::Series(ref opaint) => paint.cmp(opaint),
                    Paint::Mixed(_) => Ordering::Less,
                }
            },
            Paint::Mixed(ref paint) => {
                match *other {
                    Paint::Series(_) => Ordering::Greater,
                    Paint::Mixed(ref opaint) => paint.cmp(opaint),
                }

            }
        }
    }
}

impl<C: CharacteristicsInterface> ColouredItemInterface for Paint<C> {
    fn colour(&self) -> Colour {
        match *self {
            Paint::Series(ref paint) => paint.colour(),
            Paint::Mixed(ref paint) => paint.colour(),
        }
    }
}

impl<C: CharacteristicsInterface> BasicPaintInterface<C> for Paint<C> {
    fn name(&self) -> String {
        match *self {
            Paint::Series(ref paint) => paint.name(),
            Paint::Mixed(ref paint) => paint.name(),
        }
    }

    fn notes(&self) -> String {
        match *self {
            Paint::Series(ref paint) => paint.notes(),
            Paint::Mixed(ref paint) => paint.notes(),
        }
    }

    fn tooltip_text(&self) -> String {
        match *self {
            Paint::Series(ref paint) => paint.tooltip_text(),
            Paint::Mixed(ref paint) => paint.tooltip_text(),
        }
    }

    fn characteristics(&self) -> C {
        match *self {
            Paint::Series(ref paint) => paint.characteristics(),
            Paint::Mixed(ref paint) => paint.characteristics(),
        }
    }
}

#[derive(Debug)]
pub enum PaintError {
    AlreadyExists(String),
    MalformedText(String),
    PaintTypeMismatch,
    IOError(io::Error),
    NoSubstantiveComponents
}

#[cfg(test)]
mod tests {
    //use super::*;


    #[test]
    fn it_works() {

    }
}
