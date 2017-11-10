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

use std::convert::From;

use colour::*;
use rgb_math::rgb::*;

#[derive(Debug, PartialEq)]
pub struct ColourComponent {
    pub colour: Colour,
    pub parts: u32
}

#[derive(Debug, PartialEq)]
pub struct ColourMixer {
    rgb_sum: RGB,
    total_parts: u32,
}

impl ColourMixer {
    pub fn new() -> ColourMixer {
        ColourMixer{rgb_sum: RGB::from((0.0, 0.0, 0.0)), total_parts: 0}
    }

    pub fn reset(&mut self) {
        self.total_parts = 0;
        self.rgb_sum = RGB::from((0.0, 0.0, 0.0));
    }

    pub fn get_colour(&self) -> Option<Colour> {
        if self.total_parts > 0 {
            let rgb = self.rgb_sum / self.total_parts;
            Some(Colour::from(rgb))
        } else {
            None
        }
    }

    pub fn add(&mut self, component: &ColourComponent) {
        self.total_parts += component.parts;
        self.rgb_sum += component.colour.rgb() * component.parts;
    }
}

impl From<Vec<ColourComponent>> for ColourMixer {
    fn from(colour_components: Vec<ColourComponent>) -> ColourMixer {
        let mut colour_mixer = ColourMixer::new();
        for colour_component in colour_components.iter() {
            colour_mixer.add(colour_component);
        };
        colour_mixer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_colour_mix_test() {
        let mut colour_mixer = ColourMixer::new();
        assert_eq!(colour_mixer.get_colour(), None);
        colour_mixer.add(&ColourComponent{parts: 10, colour: Colour::from(RED)});
        assert_eq!(colour_mixer.get_colour(), Some(Colour::from(RED)));
    }
}
