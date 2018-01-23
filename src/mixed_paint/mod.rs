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

use std::cmp::Ordering;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::{StaticType, ToValue};

use pw_gix::colour::*;

use basic_paint::*;
use series_paint::*;

pub mod collection;
pub mod components;
pub mod display;
pub mod hue_wheel;
pub mod match_area;
pub mod mixer;
pub mod target;

use self::target::TargetColour;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MixingMode {
    MatchTarget,
    MatchSamples
}

#[derive(Debug, Clone, Hash)]
pub enum Paint<C: CharacteristicsInterface> {
    Series(SeriesPaint<C>),
    Mixed(MixedPaint<C>)
}

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

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct PaintComponent<C: CharacteristicsInterface> {
    pub paint: Paint<C>,
    pub parts: u32
}

pub const MP_NAME: i32 = SP_NAME;
pub const MP_NOTES: i32 = SP_NOTES;
pub const MP_CHROMA: i32 = SP_CHROMA;
pub const MP_GREYNESS: i32 = SP_GREYNESS;
pub const MP_VALUE: i32 = SP_VALUE;
pub const MP_WARMTH: i32 = SP_WARMTH;
pub const MP_RGB: i32 = SP_RGB;
pub const MP_RGB_FG: i32 = SP_RGB_FG;
pub const MP_MONO_RGB: i32 = SP_MONO_RGB;
pub const MP_MONO_RGB_FG: i32 = SP_MONO_RGB_FG;
pub const MP_WARMTH_RGB: i32 = SP_WARMTH_RGB;
pub const MP_WARMTH_RGB_FG: i32 = SP_WARMTH_RGB_FG;
pub const MP_HUE_RGB: i32 = SP_HUE_RGB;
pub const MP_HUE_ANGLE: i32 = SP_HUE_ANGLE;
pub const MP_MATCHED_RGB: i32 = 14;
pub const MP_MATCHED_ANGLE: i32 = 15;
pub const MP_CHARS_0: i32 = 16;
pub const MP_CHARS_1: i32 = 17;
pub const MP_CHARS_2: i32 = 18;
pub const MP_CHARS_3: i32 = 19;

lazy_static! {
    pub static ref MIXED_PAINT_ROW_SPEC: [gtk::Type; 20] =
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
            gdk::RGBA::static_type(),   // 14 Matched Colour
            f64::static_type(),         // 15 Matched Colour angle (radians)
            gtk::Type::String,          // 16 Characteristic #1
            gtk::Type::String,          // 17 Characteristic #2
            gtk::Type::String,          // 18 Characteristic #3
            gtk::Type::String,          // 19 Characteristic #4
        ];
}

#[derive(Debug, Clone, Hash)]
pub struct MixedPaintCore<C: CharacteristicsInterface> {
    colour: Colour,
    name: String,
    notes: String,
    characteristics: C,
    target_colour: Option<TargetColour>,
    components: Rc<Vec<PaintComponent<C>>>
}

impl<C: CharacteristicsInterface> PartialEq for MixedPaintCore<C> {
    fn eq(&self, other: &MixedPaintCore<C>) -> bool {
        self.name == other.name
    }
}

impl<C: CharacteristicsInterface> Eq for MixedPaintCore<C> {}

impl<C: CharacteristicsInterface> PartialOrd for MixedPaintCore<C> {
    fn partial_cmp(&self, other: &MixedPaintCore<C>) -> Option<Ordering> {
       self.name.partial_cmp(&other.name)
    }
}

impl<C: CharacteristicsInterface> Ord for MixedPaintCore<C> {
    fn cmp(&self, other: &MixedPaintCore<C>) -> Ordering {
       self.name.cmp(&other.name)
    }
}

impl<C: CharacteristicsInterface> MixedPaintCore<C> {
    pub fn uses_paint(&self, paint: &Paint<C>) -> bool {
        for component in self.components.iter() {
            if *paint == component.paint {
                return true;
            } else if let Paint::Mixed(ref mixed_paint) = component.paint {
                if mixed_paint.uses_paint(paint) {
                    return true
                }
            }
        };
        false
    }

    pub fn uses_series_paint(&self, paint: &SeriesPaint<C>) -> bool {
        self.uses_paint(&Paint::Series(paint.clone()))
    }

    pub fn uses_mixed_paint(&self, paint: &MixedPaint<C>) -> bool {
        self.uses_paint(&Paint::Mixed(paint.clone()))
    }

    pub fn series_paints_used(&self) -> Vec<SeriesPaint<C>> {
        let mut spu: Vec<SeriesPaint<C>> = Vec::new();
        for component in self.components.iter() {
            match component.paint {
                Paint::Series(ref series_paint) => {
                    if let Err(index) = spu.binary_search(series_paint) {
                        // NB: Ok case means paint already in the list
                        spu.insert(index, series_paint.clone())
                    }
                },
                Paint::Mixed(ref mixed_paint) => {
                    for series_paint in mixed_paint.series_paints_used().iter() {
                        if let Err(index) = spu.binary_search(series_paint) {
                            // NB: Ok case means paint already in the list
                            spu.insert(index, series_paint.clone())
                        }
                    }
                },
            }
        }

        spu
    }

    pub fn matched_colour(&self) -> Option<Colour> {
        if let Some(ref target_colour) = self.target_colour {
            Some(target_colour.colour())
        } else {
            None
        }
    }

    pub fn target_colour(&self) -> Option<TargetColour> {
        if let Some(ref target_colour) = self.target_colour {
            Some(target_colour.clone())
        } else {
            None
        }
    }

    pub fn components(&self) -> Rc<Vec<PaintComponent<C>>> {
        self.components.clone()
    }
}

pub type MixedPaint<C> = Rc<MixedPaintCore<C>>;

impl<C: CharacteristicsInterface> ColouredItemInterface for MixedPaint<C> {
    fn colour(&self) -> Colour {
        self.colour.clone()
    }
}

impl<C> BasicPaintInterface<C> for MixedPaint<C>
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn notes(&self) -> String {
        self.notes.clone()
    }

    fn tooltip_text(&self) -> String {
        format!("{}: {}", self.name, self.notes)
    }

    fn characteristics(&self) -> C {
        self.characteristics.clone()
    }

    fn tv_row_len() -> usize {
        MP_CHARS_0 as usize + C::tv_row_len()
    }

    fn tv_rows(&self) -> Vec<gtk::Value> {
        let rgba: gdk::RGBA = self.rgb().into();
        let frgba: gdk::RGBA = self.rgb().best_foreground_rgb().into();
        let mrgba: gdk::RGBA = self.monotone_rgb().into();
        let mfrgba: gdk::RGBA = self.monotone_rgb().best_foreground_rgb().into();
        let wrgba: gdk::RGBA = self.warmth_rgb().into();
        let wfrgba: gdk::RGBA = self.warmth_rgb().best_foreground_rgb().into();
        let hrgba: gdk::RGBA = self.max_chroma_rgb().into();
        let mcrgba: gdk::RGBA = if let Some(colour) = self.matched_colour() {
            colour.rgb().into()
        } else {
            self.rgb().into()
        };
        let mcsort: f64 = if let Some(colour) = self.matched_colour() {
            colour.hue().angle().radians()
        } else {
            self.hue().angle().radians()
        };
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
            mcrgba.to_value(),
            mcsort.to_value(),
        ];
        for row in self.characteristics().tv_rows().iter() {
            rows.push(row.clone());
        };
        rows
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
