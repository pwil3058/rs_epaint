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
use std::path::Path;
use std::rc::Rc;

use pw_gix::colour::*;

pub mod entry;
pub mod hue_wheel;
pub mod manager;
pub mod series;

use paint::*;

pub use self::series::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
pub struct PaintSeriesIdentityData {
    manufacturer: String,
    series_name: String,
}

impl PaintSeriesIdentityData {
    pub fn manufacturer(&self) -> String {
        self.manufacturer.clone()
    }

    pub fn series_name(&self) -> String {
        self.series_name.clone()
    }
}

pub type PaintSeriesIdentity = Rc<PaintSeriesIdentityData>;

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

impl<C: CharacteristicsInterface> ColouredItemInterface for SeriesPaint<C> {
    fn colour(&self) -> Colour {
        self.colour.clone()
    }
}

pub type SeriesPaint<C> = Rc<SeriesPaintCore<C>>;

impl<C> BasicPaintInterface<C> for SeriesPaint<C>
    where   C: CharacteristicsInterface
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn notes(&self) -> String {
        self.notes.clone()
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

impl<C: CharacteristicsInterface> PaintTreeViewColumnData<C> for SeriesPaint<C> {}

impl<C> SeriesPaintInterface<C> for SeriesPaint<C>
    where   C: CharacteristicsInterface
{
    fn series(&self) -> PaintSeriesIdentity {
        self.series_id.clone()
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

}
