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

use gtk;
use gtk::prelude::*;

use std::str::FromStr;

use colour::attributes::*;
use paint::*;
use paint::characteristics::*;
use paint::components::*;
use paint::mixer::*;
use paint::series::*;

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub struct ModelPaintCharacteristics {
    pub finish: Finish,
    pub transparency: Transparency,
    pub fluorescence: Fluorescence,
    pub metallic: Metallic,
}

impl CharacteristicsInterface for ModelPaintCharacteristics {
    fn gui_display_widget(&self) -> gtk::Box {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let label = gtk::Label::new(self.finish.description());
        vbox.pack_start(&label, false, false, 1);
        let label = gtk::Label::new(self.transparency.description());
        vbox.pack_start(&label, false, false, 1);
        let label = gtk::Label::new(self.fluorescence.description());
        vbox.pack_start(&label, false, false, 1);
        let label = gtk::Label::new(self.metallic.description());
        vbox.pack_start(&label, false, false, 1);
        vbox.show_all();
        vbox
    }
}

impl FromStr for ModelPaintCharacteristics {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<ModelPaintCharacteristics, PaintError> {
        let finish = Finish::from_str(string)?;
        let transparency = Transparency::from_str(string)?;
        let fluorescence = Fluorescence::from_str(string)?;
        let metallic = Metallic::from_str(string)?;
        Ok(ModelPaintCharacteristics{finish, transparency, fluorescence, metallic})
    }
}

pub type ModelSeriesPaint = SeriesPaint<ModelPaintCharacteristics>;
pub type ModelSeriesPaintSpec = SeriesPaintSpec<ModelPaintCharacteristics>;
pub type ModelMixedPaint = MixedPaint<ModelPaintCharacteristics>;
pub type ModelPaint = Paint<ModelPaintCharacteristics>;
pub type ModelPaintSeries = PaintSeries<ModelPaintCharacteristics>;
pub type ModelPaintComponentsBox = PaintComponentsBox<ModelPaintCharacteristics>;
pub type ModelPaintMixer = PaintMixer<HueChromaValueCADS, ModelPaintCharacteristics>;

const IDEAL_PAINT_STR: &str =
"Manufacturer: Imaginary
Series: Ideal Paint Colours Series
ModelPaint(name=\"Black\", rgb=RGB16(red=0x0, green=0x0, blue=0x0), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Blue\", rgb=RGB16(red=0x0, green=0x0, blue=0xFFFF), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Cyan\", rgb=RGB16(red=0x0, green=0xFFFF, blue=0xFFFF), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Green\", rgb=RGB16(red=0x0, green=0xFFFF, blue=0x0), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Magenta\", rgb=RGB16(red=0xFFFF, green=0x0, blue=0xFFFF), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Red\", rgb=RGB16(red=0xFFFF, green=0x0, blue=0x0), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"White\", rgb=RGB16(red=0xFFFF, green=0xFFFF, blue=0xFFFF), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")
ModelPaint(name=\"Yellow\", rgb=RGB16(red=0xFFFF, green=0xFFFF, blue=0x0), transparency=\"O\", finish=\"G\", metallic=\"NM\", fluorescence=\"NF\", notes=\"\")";

pub fn create_ideal_model_paint_series() -> ModelPaintSeries {
    ModelPaintSeries::from_str(IDEAL_PAINT_STR).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_model_paint() {
        let test_str = r#"ModelPaint(name="71.001 White", rgb=RGB16(red=0xF800, green=0xFA00, blue=0xF600), transparency="O", finish="F", metallic="NM", fluorescence="NF", notes="FS37925 RAL9016 RLM21")"#.to_string();
        assert!(SERIES_PAINT_RE.is_match(&test_str));
        if let Ok(spec) = ModelSeriesPaintSpec::from_str(&test_str) {
            assert_eq!(spec.name, "71.001 White");
            assert_eq!(spec.characteristics.finish, Finish::Flat);
            assert_eq!(spec.characteristics.transparency, Transparency::Opaque);
            assert_eq!(spec.characteristics.fluorescence, Fluorescence::Nonfluorescent);
            assert_eq!(spec.characteristics.metallic, Metallic::Nonmetallic);
            assert_eq!(spec.notes, "FS37925 RAL9016 RLM21");
            let rgb16 = RGB16::from(spec.rgb);
            assert_eq!(rgb16.red, u16::from_str_radix("F800", 16).unwrap());
            assert_eq!(rgb16.green, u16::from_str_radix("FA00", 16).unwrap());
            assert_eq!(rgb16.blue, u16::from_str_radix("F600", 16).unwrap());
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    #[test]
    fn paint_model_paint_ideal_series() {
        if let Ok(series) = ModelPaintSeries::from_str(IDEAL_PAINT_STR) {
            for pair in [
                ("Red", RED),
                ("Green", GREEN),
                ("Blue", BLUE),
                ("Cyan", CYAN),
                ("Magenta", MAGENTA),
                ("Yellow", YELLOW),
                ("Black", BLACK),
                ("White", WHITE)
            ].iter()
            {
                assert_eq!(series.get_series_paint(pair.0).unwrap().colour().rgb(), pair.1);
            }
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
        let series = create_ideal_model_paint_series();
        for pair in [
            ("Red", RED),
            ("Green", GREEN),
            ("Blue", BLUE),
            ("Cyan", CYAN),
            ("Magenta", MAGENTA),
            ("Yellow", YELLOW),
            ("Black", BLACK),
            ("White", WHITE)
        ].iter()
        {
            assert_eq!(series.get_series_paint(pair.0).unwrap().colour().rgb(), pair.1);
            assert_eq!(series.get_paint(pair.0).unwrap().colour().rgb(), pair.1);
        }
    }
}
