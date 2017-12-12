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

use pw_gix::colour::*;

use basic_paint::*;
use mixed_paint::*;
use series_paint::*;

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

#[cfg(test)]
mod tests {
    //use super::*;


    #[test]
    fn it_works() {

    }
}
