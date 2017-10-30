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

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::*;
//use std::ops::Index;
use std::rc::Rc;
//use std::slice::Iter;
use std::str::FromStr;

use regex::Regex;

use colour::*;
use rgb_math::rgb::*;

pub mod characteristics;
pub mod colour_mix;
pub mod components;
pub mod hue_wheel;
pub mod mixer;
pub mod model_paint;
//pub mod series;
pub mod shape;
pub mod target;


pub trait CharacteristicsInterface:
    Debug + Hash + PartialEq + Clone + Copy + FromStr
{

}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
pub struct PaintSeriesIdentityData {
    manufacturer: String,
    series_name: String,
}

pub type PaintSeriesIdentity = Rc<PaintSeriesIdentityData>;

pub trait BasicPaintInterface<C>: Hash + Clone + PartialEq
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String;
    fn colour(&self) -> Colour;
    fn notes(&self) -> String;
    fn tooltip_text(&self) -> String;
    fn characteristics(&self) -> C;
}

pub trait SeriesPaintInterface<C>: BasicPaintInterface<C>
    where   C: CharacteristicsInterface
{
    fn series(&self) -> PaintSeriesIdentity;
}

#[derive(Debug, Hash, Clone)]
pub struct SeriesPaintCore<C: CharacteristicsInterface> {
    colour: Colour,
    name: String,
    notes: String,
    characteristics: C,
    series_id: PaintSeriesIdentity
}

impl<C: CharacteristicsInterface> PartialEq for SeriesPaintCore<C> {
    fn eq(&self, other: &SeriesPaintCore<C>) -> bool {
        if self.series_id != other.series_id {
            false
        } else {
            self.name == other.name
        }
    }
}

impl<C: CharacteristicsInterface> Eq for SeriesPaintCore<C> {}

impl<C: CharacteristicsInterface> PartialOrd for SeriesPaintCore<C> {
    fn partial_cmp(&self, other: &SeriesPaintCore<C>) -> Option<Ordering> {
        if let Some(ordering) = self.series_id.partial_cmp(&other.series_id) {
            if ordering == Ordering::Equal {
                self.name.partial_cmp(&other.name)
            } else {
                Some(ordering)
            }
        } else {
            //panic!("File: {:?} Line: {:?}", file!(), line!())
            None
        }
    }
}

impl<C: CharacteristicsInterface> Ord for SeriesPaintCore<C> {
    fn cmp(&self, other: &SeriesPaintCore<C>) -> Ordering {
        let ordering = self.series_id.cmp(&other.series_id);
        if ordering == Ordering::Equal {
            self.name.cmp(&other.name)
        } else {
            ordering
        }
    }
}

pub type SeriesPaint<C> = Rc<SeriesPaintCore<C>>;

impl<C> BasicPaintInterface<C> for SeriesPaint<C>
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn colour(&self) -> Colour {
        self.colour.clone()
    }

    fn notes(&self) -> String {
        self.name.clone()
    }

    fn tooltip_text(&self) -> String {
        if self.notes.len() > 0 {
            format!(
                "{} ({})\n{}\n{}",
                self.series_id.series_name, self.series_id.manufacturer,
                self.name, self.notes
            )
        } else {
            format!(
                "{}: {}\n{}",
                self.series_id.manufacturer, self.series_id.series_name,
                self.name
            )
        }
    }

    fn characteristics(&self) -> C {
        self.characteristics.clone()
    }
}

impl<C> SeriesPaintInterface<C> for SeriesPaint<C>
    where   C: CharacteristicsInterface
{
    fn series(&self) -> PaintSeriesIdentity {
        self.series_id.clone()
    }
}

pub trait MixedPaintInterface<C: CharacteristicsInterface> {
    fn components(&self) -> Rc<Vec<PaintComponent<C>>>;
}

#[derive(Debug, Clone, Hash)]
pub struct MixedPaintCore<C: CharacteristicsInterface> {
    colour: Colour,
    name: String,
    notes: String,
    characteristics: C,
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

pub type MixedPaint<C> = Rc<MixedPaintCore<C>>;

impl<C> BasicPaintInterface<C> for MixedPaint<C>
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn colour(&self) -> Colour {
        self.colour.clone()
    }

    fn notes(&self) -> String {
        self.name.clone()
    }

    fn tooltip_text(&self) -> String {
        format!("{}: {}", self.name, self.notes)
    }

    fn characteristics(&self) -> C {
        self.characteristics.clone()
    }
}

impl<C> MixedPaintInterface<C> for MixedPaint<C>
    where   C: CharacteristicsInterface
{
    fn components(&self) -> Rc<Vec<PaintComponent<C>>> {
        self.components.clone()
    }
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

impl<C: CharacteristicsInterface> BasicPaintInterface<C> for Paint<C> {
    fn name(&self) -> String {
        match *self {
            Paint::Series(ref paint) => paint.name(),
            Paint::Mixed(ref paint) => paint.name(),
        }
    }

    fn colour(&self) -> Colour {
        match *self {
            Paint::Series(ref paint) => paint.colour(),
            Paint::Mixed(ref paint) => paint.colour(),
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
    paint: Paint<C>,
    parts: u32
}

lazy_static! {
    pub static ref MANUFACTURER_RE: Regex = Regex::new(
        r#"^Manufacturer:\s*(?P<manufacturer>.*)\s*$"#
    ).unwrap();

    pub static ref SERIES_RE: Regex = Regex::new(
        r#"^Series:\s*(?P<series>.*)\s*$"#
    ).unwrap();

    pub static ref SERIES_PAINT_RE: Regex = Regex::new(
        r#"^(?P<ptype>\w+)\(name="(?P<name>.+)", rgb=(?P<rgb>RGB16\([^)]+\)), (?P<characteristics>(?:\w+="\w+", )*)notes="(?P<notes>.*)"\)$"#
    ).unwrap();
}

fn manufacturer_from_str(string: &str) -> Result<String, PaintError> {
    if let Some(captures) = MANUFACTURER_RE.captures(string) {
        match captures.name("manufacturer") {
            Some(m_name) => return Ok(m_name.as_str().to_string()),
            None => return Err(PaintError::MalformedText(string.to_string()))
        }
    } else {
        return Err(PaintError::MalformedText(string.to_string()));
    }
}

fn series_from_str(string: &str) -> Result<String, PaintError> {
    if let Some(captures) = SERIES_RE.captures(string) {
        match captures.name("series") {
            Some(m_name) => return Ok(m_name.as_str().to_string()),
            None => return Err(PaintError::MalformedText(string.to_string()))
        }
    } else {
        return Err(PaintError::MalformedText(string.to_string()));
    }
}

#[derive(Debug, PartialEq)]
pub struct SeriesPaintSpec<C: CharacteristicsInterface> {
    rgb: RGB,
    name: String,
    notes: String,
    characteristics: C,
}

impl<C: CharacteristicsInterface> FromStr for SeriesPaintSpec<C> {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<SeriesPaintSpec<C>, PaintError> {
        let captures = SERIES_PAINT_RE.captures(string).ok_or(PaintError::MalformedText(string.to_string()))?;
        let c_match = captures.name("characteristics").ok_or(PaintError::MalformedText(string.to_string()))?;
        let rgb_match = captures.name("rgb").ok_or(PaintError::MalformedText(string.to_string()))?;
        let notes_match = captures.name("notes").ok_or(PaintError::MalformedText(string.to_string()))?;
        let name_match = captures.name("name").ok_or(PaintError::MalformedText(string.to_string()))?;
        let characteristics = C::from_str(c_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        let rgb16 = RGB16::from_str(rgb_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        Ok(
            SeriesPaintSpec::<C> {
                rgb: RGB::from(rgb16),
                name: name_match.as_str().to_string(),
                notes: notes_match.as_str().to_string(),
                characteristics: characteristics,
            }
        )
    }
}

#[derive(Debug)]
pub enum PaintError {
    AlreadyExists(String),
    MalformedText(String),
    PaintTypeMismatch,
}

pub struct PaintSeriesCore<C: CharacteristicsInterface> {
    series_id: PaintSeriesIdentity,
    paints: RefCell<Vec<SeriesPaint<C>>>
}

impl<C: CharacteristicsInterface> FromStr for PaintSeriesCore<C> {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<PaintSeriesCore<C>, PaintError> {
        let mut lines = string.lines();
        let manufacturer = match lines.next() {
            Some(line) => {
                manufacturer_from_str(line)?
            },
            None => return Err(PaintError::MalformedText(string.to_string())),
        };
        let series_name = match lines.next() {
            Some(line) => {
                series_from_str(line)?
            },
            None => return Err(PaintError::MalformedText(string.to_string())),
        };
        let series_id = Rc::new(PaintSeriesIdentityData{manufacturer, series_name});
        let paints: RefCell<Vec<SeriesPaint<C>>> = RefCell::new(Vec::new());
        let psc = PaintSeriesCore::<C>{series_id, paints};
        for line in lines {
            let spec = SeriesPaintSpec::<C>::from_str(line)?;
            psc.add_paint(&spec)?;
        }
        Ok(psc)
    }
}

impl<C: CharacteristicsInterface> PaintSeriesCore<C> {
    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints.borrow().binary_search_by_key(
            &name.to_string(),
            |paint| paint.name()
        )
    }

    pub fn series_id(&self) -> PaintSeriesIdentity {
        self.series_id.clone()
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<Paint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(Paint::Series(self.paints.borrow()[index].clone())),
            Err(_) => None
        }
    }

    pub fn get_series_paint(&self, name: &str) -> Option<SeriesPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }

    pub fn add_paint(&self, spec: &SeriesPaintSpec<C>) -> Result<SeriesPaint<C>, PaintError> {
        match self.find_name(&spec.name) {
            Ok(_) => Err(PaintError::AlreadyExists(spec.name.clone())),
            Err(index) => {
                let paint = Rc::new(
                    SeriesPaintCore::<C> {
                        name: spec.name.clone(),
                        notes: spec.notes.clone(),
                        colour: Colour::from(spec.rgb),
                        characteristics: spec.characteristics.clone(),
                        series_id: self.series_id.clone()
                    }
                );
                self.paints.borrow_mut().insert(index, paint.clone());
                Ok(paint)
            }
        }
    }
}

pub type PaintSeries<C> = Rc<PaintSeriesCore<C>>;

pub trait PaintSeriesInterface<C: CharacteristicsInterface> {
    fn create(manufacturer: &str, series: &str) -> PaintSeries<C>;

    fn from_str(string: &str) -> Result<PaintSeries<C>, PaintError> {
        let core = PaintSeriesCore::<C>::from_str(string)?;
        Ok(Rc::new(core))
    }
}

impl<C> PaintSeriesInterface<C> for PaintSeries<C>
    where   C: CharacteristicsInterface
{
    fn create(manufacturer: &str, series_name: &str) -> PaintSeries<C> {
        let manufacturer = manufacturer.to_string();
        let series_name = series_name.to_string();
        let series_id = Rc::new(PaintSeriesIdentityData{manufacturer, series_name});
        let paints: RefCell<Vec<SeriesPaint<C>>> = RefCell::new(Vec::new());
        Rc::new(PaintSeriesCore::<C>{series_id, paints})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_series_paint_regex() {
        let test_str = r#"ModelPaint(name="71.001 White", rgb=RGB16(red=0xF800, green=0xFA00, blue=0xF600), transparency="O", finish="F", metallic="NM", fluorescence="NF", notes="FS37925 RAL9016 RLM21")"#.to_string();
        assert!(SERIES_PAINT_RE.is_match(&test_str));
        let captures = SERIES_PAINT_RE.captures(&test_str).unwrap();
        assert_eq!(captures.name("ptype").unwrap().as_str(), "ModelPaint");
        assert_eq!(captures.name("rgb").unwrap().as_str(), "RGB16(red=0xF800, green=0xFA00, blue=0xF600)");
        assert_eq!(captures.name("characteristics").unwrap().as_str(), "transparency=\"O\", finish=\"F\", metallic=\"NM\", fluorescence=\"NF\", ");
        assert_eq!(captures.name("notes").unwrap().as_str(), "FS37925 RAL9016 RLM21");
    }
}
