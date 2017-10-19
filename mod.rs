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

use std::hash::*;
//use std::ops::Index;
use std::rc::Rc;
//use std::slice::Iter;

use colour::*;

pub mod colour_mix;
pub mod components;
pub mod mixer;
//pub mod model_paint;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, Hash)]
pub struct PaintSeriesIdentityData {
    manufacturer: String,
    series_name: String,
}

pub type PaintSeriesIdentity = Rc<PaintSeriesIdentityData>;

pub trait BasicPaintInterface<C>: Hash + Clone + PartialEq
    where   C: Hash + Clone + PartialEq + Copy
{
    fn name(&self) -> String;
    fn colour(&self) -> Colour;
    fn notes(&self) -> String;
    fn characteristics(&self) -> C;
}

pub trait SeriesPaintInterface<C>: BasicPaintInterface<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn series(&self) -> PaintSeriesIdentity;
}

#[derive(Debug, Hash)]
pub struct SeriesPaintCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    colour: Colour,
    name: String,
    notes: String,
    characteristics: C,
    series_id: PaintSeriesIdentity
}

impl<C> PartialEq for SeriesPaintCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn eq(&self, other: &SeriesPaintCore<C>) -> bool {
        if self.series_id != other.series_id {
            false
        } else {
            self.name == other.name
        }
    }
}

pub type SeriesPaint<C> = Rc<SeriesPaintCore<C>>;

impl<C> BasicPaintInterface<C> for SeriesPaint<C>
    where   C: Hash + Clone + PartialEq + Copy
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

    fn characteristics(&self) -> C {
        self.characteristics
    }
}

impl<C> SeriesPaintInterface<C> for SeriesPaint<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn series(&self) -> PaintSeriesIdentity {
        self.series_id.clone()
    }
}

pub trait MixedPaintInterface<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn components(&self) -> Rc<Vec<PaintComponent<C>>>;
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct MixedPaintCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    colour: Colour,
    name: String,
    notes: String,
    characteristics: C,
    components: Rc<Vec<PaintComponent<C>>>
}

pub type MixedPaint<C> = Rc<MixedPaintCore<C>>;

impl<C> BasicPaintInterface<C> for MixedPaint<C>
    where   C: Hash + Clone + PartialEq + Copy
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

    fn characteristics(&self) -> C {
        self.characteristics
    }
}

impl<C> MixedPaintInterface<C> for MixedPaint<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn components(&self) -> Rc<Vec<PaintComponent<C>>> {
        self.components.clone()
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Paint<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    Series(SeriesPaint<C>),
    Mixed(MixedPaint<C>)
}

impl<C> BasicPaintInterface<C> for Paint<C>
    where   C: Hash + Clone + PartialEq + Copy
{
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

    fn characteristics(&self) -> C {
        match *self {
            Paint::Series(ref paint) => paint.characteristics(),
            Paint::Mixed(ref paint) => paint.characteristics(),
        }
    }
}

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct PaintComponent<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    paint: Paint<C>,
    parts: u32
}

pub trait PaintSeriesInterface<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn series_id(&self) -> PaintSeriesIdentity;
    fn len(&self) -> usize;
    fn get_paint(&self, name: &str) -> Option<Paint<C>>;
    fn get_series_paint(&self, name: &str) -> Option<SeriesPaint<C>>;
    fn has_paint_named(&self, name: &str) -> bool;
    //fn iter(&'a self) -> Iter<'a, P>;
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
