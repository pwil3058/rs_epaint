// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::convert::From;

use crate::colour::*;

#[derive(Debug, PartialEq)]
pub struct ColourComponent {
    pub colour: Colour,
    pub parts: u32,
}

#[derive(Debug, PartialEq)]
pub struct ColourMixer {
    rgb_sum: [f64; 3],
    total_parts: u32,
}

impl ColourMixer {
    pub fn new() -> ColourMixer {
        ColourMixer {
            rgb_sum: [0.0, 0.0, 0.0],
            total_parts: 0,
        }
    }

    pub fn reset(&mut self) {
        self.total_parts = 0;
        self.rgb_sum = [0.0, 0.0, 0.0];
    }

    pub fn get_colour(&self) -> Option<Colour> {
        if self.total_parts > 0 {
            let divisor = self.total_parts as f64;
            let array: [f64; 3] = [
                self.rgb_sum[0] / divisor,
                self.rgb_sum[1] / divisor,
                self.rgb_sum[2] / divisor,
            ];
            Some(Colour::from(RGB::from(array)))
        } else {
            None
        }
    }

    pub fn add(&mut self, colour: &Colour, parts: u32) {
        self.total_parts += parts;
        self.rgb_sum[0] += colour.rgb()[CCI::Red] * parts as f64;
        self.rgb_sum[1] += colour.rgb()[CCI::Green] * parts as f64;
        self.rgb_sum[2] += colour.rgb()[CCI::Blue] * parts as f64;
    }
}

impl From<Vec<(Colour, u32)>> for ColourMixer {
    fn from(colour_components: Vec<(Colour, u32)>) -> ColourMixer {
        let mut colour_mixer = ColourMixer::new();
        for (ref colour, parts) in colour_components {
            colour_mixer.add(colour, parts);
        }
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
        colour_mixer.add(&Colour::from(RGB::RED), 10);
        assert_eq!(colour_mixer.get_colour(), Some(Colour::from(RGB::RED)));
    }
}
