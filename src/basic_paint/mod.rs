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

use std::fmt::Debug;
use std::hash::*;
use std::rc::Rc;
use std::str::FromStr;

use gdk;
use gtk;
use gtk::StaticType;
use gtk::prelude::*;

use regex::*;

use pw_gix::colour::*;
use pw_gix::rgb_math::rgb::*;
use pw_gix::rgb_math::hue::*;

use error::*;

pub mod collection;
//pub mod editor;
pub mod entry;
pub mod factory;
pub mod hue_wheel;

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

pub trait BasicPaintInterface<C>: Hash + Clone + PartialEq + Ord + Debug + ColouredItemInterface
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String;
    fn notes(&self) -> String;
    fn tooltip_text(&self) -> String;
    fn characteristics(&self) -> C;

    fn get_spec(&self) -> BasicPaintSpec<C> {
        BasicPaintSpec::<C> {
            rgb: self.rgb(),
            name: self.name(),
            notes: self.notes(),
            characteristics: self.characteristics(),
        }
    }

    fn matches_spec(&self, spec: BasicPaintSpec<C>) -> bool {
        if self.rgb() != spec.rgb {
            false
        } else if self.name() != spec.name {
            false
        } else if self.notes() != spec.notes {
            false
        } else if self.characteristics() != spec.characteristics {
            false
        } else {
            true
        }
    }

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

#[derive(Debug, PartialEq)]
pub struct BasicPaintSpec<C: CharacteristicsInterface> {
    pub rgb: RGB,
    pub name: String,
    pub notes: String,
    pub characteristics: C,
}

pub trait CreateFromSpec<C: CharacteristicsInterface> {
    fn create(spec: &BasicPaintSpec<C>) -> Self;
}

lazy_static! {
    pub static ref BASIC_PAINT_RE: Regex = Regex::new(
        r#"^(?P<ptype>\w+)\((name=)?"(?P<name>.+)", rgb=(?P<rgb>RGB(16)?\([^)]+\))(?P<characteristics>(?:, \w+="\w+")*)(, notes="(?P<notes>.*)")?\)$"#
    ).unwrap();
}

impl<C: CharacteristicsInterface> FromStr for BasicPaintSpec<C> {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<BasicPaintSpec<C>, PaintError> {
        let captures = BASIC_PAINT_RE.captures(string).ok_or(PaintError::MalformedText(string.to_string()))?;
        let c_match = captures.name("characteristics").ok_or(PaintError::MalformedText(string.to_string()))?;
        let rgb_match = captures.name("rgb").ok_or(PaintError::MalformedText(string.to_string()))?;
        let name_match = captures.name("name").ok_or(PaintError::MalformedText(string.to_string()))?;
        let characteristics = C::from_str(c_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        let rgb16 = RGB16::from_str(rgb_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        let notes = match captures.name("notes") {
            Some(notes_match) => notes_match.as_str().to_string(),
            None => "".to_string()
        };
        Ok(
            BasicPaintSpec::<C> {
                rgb: RGB::from(rgb16),
                name: name_match.as_str().to_string(),
                notes: notes,
                characteristics: characteristics,
            }
        )
    }
}

pub const SP_NAME: i32 = 0;
pub const SP_NOTES: i32 = 1;
pub const SP_CHROMA: i32 = 2;
pub const SP_GREYNESS: i32 = 3;
pub const SP_VALUE: i32 = 4;
pub const SP_WARMTH: i32 = 5;
pub const SP_RGB: i32 = 6;
pub const SP_RGB_FG: i32 = 7;
pub const SP_MONO_RGB: i32 = 8;
pub const SP_MONO_RGB_FG: i32 = 9;
pub const SP_WARMTH_RGB: i32 = 10;
pub const SP_WARMTH_RGB_FG: i32 = 11;
pub const SP_HUE_RGB: i32 = 12;
pub const SP_HUE_ANGLE: i32 = 13;
pub const SP_CHARS_0: i32 = 14;
pub const SP_CHARS_1: i32 = 15;
pub const SP_CHARS_2: i32 = 16;
pub const SP_CHARS_3: i32 = 17;

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

pub trait PaintTreeViewColumnSpec {
    fn tv_columns() -> Vec<gtk::TreeViewColumn>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_paint_basic_paint_regex() {
        let test_str = r#"ModelPaint(name="71.001 White", rgb=RGB16(red=0xF800, green=0xFA00, blue=0xF600), transparency="O", finish="F", metallic="NM", fluorescence="NF", notes="FS37925 RAL9016 RLM21")"#.to_string();
        assert!(BASIC_PAINT_RE.is_match(&test_str));
        let captures = BASIC_PAINT_RE.captures(&test_str).unwrap();
        assert_eq!(captures.name("ptype").unwrap().as_str(), "ModelPaint");
        assert_eq!(captures.name("rgb").unwrap().as_str(), "RGB16(red=0xF800, green=0xFA00, blue=0xF600)");
        assert_eq!(captures.name("characteristics").unwrap().as_str(), ", transparency=\"O\", finish=\"F\", metallic=\"NM\", fluorescence=\"NF\"");
        assert_eq!(captures.name("notes").unwrap().as_str(), "FS37925 RAL9016 RLM21");
    }

    #[test]
    fn basic_paint_basic_paint_obsolete_regex() {
        let test_str = r#"NamedColour(name="XF 1: Flat Black *", rgb=RGB(0x2D00, 0x2B00, 0x3000), transparency="O", finish="F")"#.to_string();
        assert!(BASIC_PAINT_RE.is_match(&test_str));
        let captures = BASIC_PAINT_RE.captures(&test_str).unwrap();
        assert_eq!(captures.name("ptype").unwrap().as_str(), "NamedColour");
        assert_eq!(captures.name("rgb").unwrap().as_str(), "RGB(0x2D00, 0x2B00, 0x3000)");
        assert_eq!(captures.name("characteristics").unwrap().as_str(), ", transparency=\"O\", finish=\"F\"");
        assert_eq!(captures.name("notes"), None);
    }
}
