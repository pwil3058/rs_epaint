// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::prelude::*;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

use pw_gix::colour::attributes::*;
use pw_gix::gtkx::tree_view_column::*;
pub use pw_gix::wrapper::*;

use crate::basic_paint::factory::*;
use crate::basic_paint::*;
use crate::characteristics::*;
use crate::colln_paint::collection::*;
use crate::colour::*;
use crate::error::*;
use crate::mixed_paint::*;
use crate::series_paint::*;

pub use crate::basic_paint::entry::*;
pub use crate::mixed_paint::mixer::*;
pub use crate::struct_traits::SimpleCreation;

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub struct ArtPaintCharacteristics {
    pub permanence: Permanence,
    pub transparency: Transparency,
}

impl CharacteristicsInterface for ArtPaintCharacteristics {
    type Entry = ArtPaintCharacteristicsEntryCore;

    fn tv_row_len() -> usize {
        2
    }

    fn tv_columns(start_col_id: i32) -> Vec<gtk::TreeViewColumn> {
        let mut cols: Vec<gtk::TreeViewColumn> = Vec::new();
        let cfw = 30;
        cols.push(simple_text_column(
            "Pe.",
            start_col_id,
            start_col_id,
            6,
            7,
            cfw,
            false,
        ));
        cols.push(simple_text_column(
            "Tr.",
            start_col_id + 1,
            start_col_id + 1,
            6,
            7,
            cfw,
            false,
        ));
        cols
    }

    fn from_floats(floats: &Vec<f64>) -> Self {
        ArtPaintCharacteristics {
            permanence: Permanence::from(floats[0]),
            transparency: Transparency::from(floats[1]),
        }
    }

    fn tv_rows(&self) -> Vec<gtk::Value> {
        let mut rows: Vec<gtk::Value> = Vec::new();
        rows.push(self.permanence.abbrev().to_value());
        rows.push(self.transparency.abbrev().to_value());
        rows
    }

    fn gui_display_widget(&self) -> gtk::Box {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let label = gtk::Label::new(Some(self.permanence.description()));
        vbox.pack_start(&label, false, false, 1);
        let label = gtk::Label::new(Some(self.transparency.description()));
        vbox.pack_start(&label, false, false, 1);
        vbox.show_all();
        vbox
    }

    fn to_floats(&self) -> Vec<f64> {
        vec![self.permanence.into(), self.transparency.into()]
    }

    fn from_str(
        string: &str,
    ) -> Result<ArtPaintCharacteristics, PaintError<ArtPaintCharacteristics>> {
        let permanence = Permanence::from_str(string)?;
        let transparency = Transparency::from_str(string)?;
        Ok(ArtPaintCharacteristics {
            permanence,
            transparency,
        })
    }
}

impl fmt::Display for ArtPaintCharacteristics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}",
            self.permanence.to_string(),
            self.transparency.to_string(),
        )
    }
}

pub struct ArtPaintCharacteristicsEntryCore {
    grid: gtk::Grid,
    permanence_entry: PermanenceEntry,
    transparency_entry: TransparencyEntry,
    changed_callbacks: RefCell<Vec<Box<dyn Fn()>>>,
}

impl ArtPaintCharacteristicsEntryCore {
    fn inform_changed(&self) {
        for callback in self.changed_callbacks.borrow().iter() {
            callback();
        }
    }
}

impl CharacteristicsEntryInterface<ArtPaintCharacteristics> for ArtPaintCharacteristicsEntryCore {
    fn create() -> Rc<ArtPaintCharacteristicsEntryCore> {
        let cei = Rc::new(ArtPaintCharacteristicsEntryCore {
            grid: gtk::Grid::new(),
            permanence_entry: PermanenceEntry::create(),
            transparency_entry: TransparencyEntry::create(),
            changed_callbacks: RefCell::new(Vec::new()),
        });
        let cei_c = cei.clone();
        cei.permanence_entry
            .combo_box_text()
            .connect_changed(move |_| cei_c.inform_changed());
        let cei_c = cei.clone();
        cei.transparency_entry
            .combo_box_text()
            .connect_changed(move |_| cei_c.inform_changed());
        cei.permanence_entry.combo_box_text().set_hexpand(true);
        cei.transparency_entry.combo_box_text().set_hexpand(true);
        let label = gtk::Label::new(Some(Permanence::prompt().as_str()));
        label.set_halign(gtk::Align::End);
        cei.grid.attach(&label, 0, 0, 1, 1);
        cei.grid.attach_next_to(
            &cei.permanence_entry.combo_box_text(),
            Some(&label),
            gtk::PositionType::Right,
            1,
            1,
        );
        let label = gtk::Label::new(Some(Transparency::prompt().as_str()));
        label.set_halign(gtk::Align::End);
        cei.grid.attach(&label, 0, 1, 1, 1);
        cei.grid.attach_next_to(
            &cei.transparency_entry.combo_box_text(),
            Some(&label),
            gtk::PositionType::Right,
            1,
            1,
        );

        cei.grid.show_all();
        cei
    }

    fn pwo(&self) -> gtk::Grid {
        self.grid.clone()
    }

    fn get_characteristics(&self) -> Option<ArtPaintCharacteristics> {
        let permanence = if let Some(value) = self.permanence_entry.get_value() {
            value
        } else {
            return None;
        };
        let transparency = if let Some(value) = self.transparency_entry.get_value() {
            value
        } else {
            return None;
        };
        Some(ArtPaintCharacteristics {
            permanence,
            transparency,
        })
    }

    fn set_characteristics(&self, o_characteristics: Option<&ArtPaintCharacteristics>) {
        if let Some(characteristics) = o_characteristics {
            self.permanence_entry
                .set_value(Some(characteristics.permanence));
            self.transparency_entry
                .set_value(Some(characteristics.transparency));
        } else {
            self.permanence_entry.set_value(None);
            self.transparency_entry.set_value(None);
        }
    }

    fn connect_changed<F: 'static + Fn()>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback))
    }
}

pub struct ArtPaintAttributes {
    vbox: gtk::Box,
    hue_cad: HueCAD,
    chroma_cad: ChromaCAD,
    value_cad: ValueCAD,
    warmth_cad: WarmthCAD,
}

impl_widget_wrapper!(vbox: gtk::Box, ArtPaintAttributes);

impl ColourAttributesInterface for ArtPaintAttributes {
    fn create() -> Rc<ArtPaintAttributes> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let hue_cad = HueCAD::create();
        let chroma_cad = ChromaCAD::create();
        let value_cad = ValueCAD::create();
        let warmth_cad = WarmthCAD::create();
        vbox.pack_start(&hue_cad.pwo(), true, true, 0);
        vbox.pack_start(&chroma_cad.pwo(), true, true, 0);
        vbox.pack_start(&value_cad.pwo(), true, true, 0);
        vbox.pack_start(&warmth_cad.pwo(), true, true, 0);
        Rc::new(ArtPaintAttributes {
            vbox,
            hue_cad,
            chroma_cad,
            value_cad,
            warmth_cad,
        })
    }

    fn tv_columns() -> Vec<gtk::TreeViewColumn> {
        let fw = 60;
        vec![
            simple_text_column("Hue", -1, SP_HUE_ANGLE, SP_HUE_RGB, -1, 50, false),
            simple_text_column(
                "Chroma",
                SP_GREYNESS,
                SP_GREYNESS,
                SP_RGB,
                SP_RGB_FG,
                fw,
                false,
            ),
            simple_text_column(
                "Value",
                SP_VALUE,
                SP_VALUE,
                SP_MONO_RGB,
                SP_MONO_RGB_FG,
                fw,
                false,
            ),
            simple_text_column(
                "Warmth",
                SP_WARMTH,
                SP_WARMTH,
                SP_WARMTH_RGB,
                SP_WARMTH_RGB_FG,
                fw,
                false,
            ),
        ]
    }

    fn scalar_attributes() -> Vec<ScalarAttribute> {
        vec![
            ScalarAttribute::Value,
            ScalarAttribute::Chroma,
            ScalarAttribute::Warmth,
        ]
    }

    fn set_colour(&self, colour: Option<&Colour>) {
        self.hue_cad.set_colour(colour);
        self.chroma_cad.set_colour(colour);
        self.value_cad.set_colour(colour);
        self.warmth_cad.set_colour(colour);
    }

    fn set_target_colour(&self, target_colour: Option<&Colour>) {
        self.hue_cad.set_target_colour(target_colour);
        self.chroma_cad.set_target_colour(target_colour);
        self.value_cad.set_target_colour(target_colour);
        self.warmth_cad.set_target_colour(target_colour);
    }
}

pub struct ArtPaintMixerConfig;

impl MixerConfig for ArtPaintMixerConfig {
    fn mixing_mode() -> MixingMode {
        MixingMode::MatchSamples
    }
}

pub type ArtSeriesPaint = SeriesPaint<ArtPaintCharacteristics>;
pub type ArtSeriesPaintSpec = BasicPaintSpec<ArtPaintCharacteristics>;
pub type ArtMixedPaint = MixedPaint<ArtPaintCharacteristics>;
pub type ArtPaint = Paint<ArtPaintCharacteristics>;
pub type BasicArtPaint = BasicPaint<ArtPaintCharacteristics>;
pub type ArtPaintSeries = SeriesPaintColln<ArtPaintCharacteristics>;
pub type ArtPaintComponentsBox =
    SeriesPaintComponentBox<ArtPaintAttributes, ArtPaintCharacteristics>;
pub type ArtPaintMixer =
    PaintMixer<ArtPaintAttributes, ArtPaintCharacteristics, ArtPaintMixerConfig>;
pub type ArtPaintSeriesManager = SeriesPaintManager<ArtPaintAttributes, ArtPaintCharacteristics>;
pub type ArtPaintFactoryDisplay =
    BasicPaintFactoryDisplay<ArtPaintAttributes, ArtPaintCharacteristics>;
pub type BasicArtPaintEditor = SeriesPaintEditor<ArtPaintAttributes, ArtPaintCharacteristics>;
pub type ArtPaintSeriesSpec = SeriesPaintCollnSpec<ArtPaintCharacteristics>;

const IDEAL_PAINT_STR: &str =
"Manufacturer: Imaginary
Series: Ideal Paint Colours Series
ArtPaint(name=\"Black\", rgb=RGB16(red=0x0, green=0x0, blue=0x0), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Blue\", rgb=RGB16(red=0x0, green=0x0, blue=0xFFFF), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Cyan\", rgb=RGB16(red=0x0, green=0xFFFF, blue=0xFFFF), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Green\", rgb=RGB16(red=0x0, green=0xFFFF, blue=0x0), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Magenta\", rgb=RGB16(red=0xFFFF, green=0x0, blue=0xFFFF), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Red\", rgb=RGB16(red=0xFFFF, green=0x0, blue=0x0), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"White\", rgb=RGB16(red=0xFFFF, green=0xFFFF, blue=0xFFFF), transparency=\"O\", permanence=\"A\", notes=\"\")
ArtPaint(name=\"Yellow\", rgb=RGB16(red=0xFFFF, green=0xFFFF, blue=0x0), transparency=\"O\", permanence=\"A\", notes=\"\")";

pub fn create_ideal_art_paint_series() -> ArtPaintSeries {
    let spec = ArtPaintSeriesSpec::from_str(IDEAL_PAINT_STR).unwrap();
    ArtPaintSeries::from_spec(&spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    //    const OBSOLETE_PAINT_STR: &str =
    //"Manufacturer: Tamiya
    //Series: Flat Acrylic (Peter Williams Digital Samples #3)
    //NamedColour(name=\"XF 1: Flat Black *\", rgb=RGB(0x2D00, 0x2B00, 0x3000), transparency=\"O\", permanence=\"C\")
    //NamedColour(name=\"XF 2: Flat White *\", rgb=RGB(0xFE00, 0xFE00, 0xFE00), transparency=\"O\", permanence=\"C\")
    //NamedColour(name=\"XF 3: Flat Yellow *\", rgb=RGB(0xF800, 0xCD00, 0x2900), transparency=\"O\", permanence=\"C\")
    //NamedColour(name=\"XF 4: Yellow Green *\", rgb=RGB(0xAA00, 0xAE00, 0x4000), transparency=\"O\", permanence=\"C\")
    //";

    #[test]
    fn art_paint() {
        let test_str = r#"ArtPaint(name="71.001 White", rgb=RGB16(red=0xF800, green=0xFA00, blue=0xF600), transparency="O", permanence="A", metallic="NM", fluorescence="NF", notes="FS37925 RAL9016 RLM21")"#.to_string();
        assert!(BASIC_PAINT_RE.is_match(&test_str));
        if let Ok(spec) = ArtSeriesPaintSpec::from_str(&test_str) {
            assert_eq!(spec.name, "71.001 White");
            assert_eq!(spec.characteristics.permanence, Permanence::Permanent);
            assert_eq!(spec.characteristics.transparency, Transparency::Opaque);
            assert_eq!(spec.notes, "FS37925 RAL9016 RLM21");
            let rgb16 = RGB16::from(spec.rgb);
            assert_eq!(rgb16[0], u16::from_str_radix("F800", 16).unwrap());
            assert_eq!(rgb16[1], u16::from_str_radix("FA00", 16).unwrap());
            assert_eq!(rgb16[2], u16::from_str_radix("F600", 16).unwrap());
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    #[test]
    fn art_paint_obsolete() {
        let test_str = r#"NamedColour(name="XF 2: Flat White *", rgb=RGB16(0xF800, 0xFA00, 0xF600), transparency="O", permanence="A")"#.to_string();
        assert!(BASIC_PAINT_RE.is_match(&test_str));
        if let Ok(spec) = ArtSeriesPaintSpec::from_str(&test_str) {
            assert_eq!(spec.name, "XF 2: Flat White *");
            assert_eq!(spec.characteristics.permanence, Permanence::Permanent);
            assert_eq!(spec.characteristics.transparency, Transparency::Opaque);
            assert_eq!(spec.notes, "");
            let rgb16 = RGB16::from(spec.rgb);
            assert_eq!(rgb16[0], u16::from_str_radix("F800", 16).unwrap());
            assert_eq!(rgb16[1], u16::from_str_radix("FA00", 16).unwrap());
            assert_eq!(rgb16[2], u16::from_str_radix("F600", 16).unwrap());
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    //    #[test]
    //    fn art_paint_ideal_series() {
    //        if let Ok(series) = ArtPaintSeries::from_str(IDEAL_PAINT_STR) {
    //            for pair in [
    //                ("Red", RED),
    //                ("Green", GREEN),
    //                ("Blue", BLUE),
    //                ("Cyan", CYAN),
    //                ("Magenta", MAGENTA),
    //                ("Yellow", YELLOW),
    //                ("Black", BLACK),
    //                ("White", WHITE),
    //            ]
    //            .iter()
    //            {
    //                assert_eq!(
    //                    series.get_series_paint(pair.0).unwrap().colour().rgb(),
    //                    pair.1
    //                );
    //            }
    //        } else {
    //            panic!("File: {:?} Line: {:?}", file!(), line!())
    //        }
    //        let series = create_ideal_art_paint_series();
    //        for pair in [
    //            ("Red", RED),
    //            ("Green", GREEN),
    //            ("Blue", BLUE),
    //            ("Cyan", CYAN),
    //            ("Magenta", MAGENTA),
    //            ("Yellow", YELLOW),
    //            ("Black", BLACK),
    //            ("White", WHITE),
    //        ]
    //        .iter()
    //        {
    //            assert_eq!(
    //                series.get_series_paint(pair.0).unwrap().colour().rgb(),
    //                pair.1
    //            );
    //            assert_eq!(series.get_paint(pair.0).unwrap().colour().rgb(), pair.1);
    //        }
    //    }
    //
    //    #[test]
    //    fn art_paint_obsolete_series() {
    //        match ArtPaintSeries::from_str(OBSOLETE_PAINT_STR) {
    //            Ok(series) => {
    //                for pair in [
    //                    (
    //                        "XF 1: Flat Black *",
    //                        RGB16::from_str("RGB(0x2D00, 0x2B00, 0x3000)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 2: Flat White *",
    //                        RGB16::from_str("RGB(0xFE00, 0xFE00, 0xFE00)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 3: Flat Yellow *",
    //                        RGB16::from_str("RGB(0xF800, 0xCD00, 0x2900)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 4: Yellow Green *",
    //                        RGB16::from_str("RGB(0xAA00, 0xAE00, 0x4000)").unwrap(),
    //                    ),
    //                ]
    //                .iter()
    //                {
    //                    assert_eq!(
    //                        series.get_series_paint(pair.0).unwrap().colour().rgb(),
    //                        RGB::from(pair.1)
    //                    );
    //                }
    //            }
    //            Err(err) => panic!("File: {:?} Line: {:?} {:?}", file!(), line!(), err),
    //        }
    //    }

    //    #[test]
    //    fn art_paint_contributions_box() {
    //        if !gtk::is_initialized() {
    //            if let Err(err) = gtk::init() {
    //                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
    //            };
    //        }
    //
    //        let components_box = ArtPaintComponentsBox::create_with(6, true);
    //        let series = create_ideal_art_paint_series();
    //        for pair in [
    //            ("Red", RED),
    //            ("Green", GREEN),
    //            ("Blue", BLUE),
    //            ("Cyan", CYAN),
    //            ("Magenta", MAGENTA),
    //            ("Yellow", YELLOW),
    //            ("Black", BLACK),
    //            ("White", WHITE),
    //        ]
    //        .iter()
    //        {
    //            assert_eq!(
    //                series.get_series_paint(pair.0).unwrap().colour().rgb(),
    //                pair.1
    //            );
    //            assert_eq!(series.get_paint(pair.0).unwrap().colour().rgb(), pair.1);
    //            let paint = series.get_paint(pair.0).unwrap();
    //            assert_eq!(paint.colour().rgb(), pair.1);
    //            components_box.add_paint(&paint);
    //        }
    //    }

    #[test]
    fn art_paint_spec_ideal_series() {
        if let Ok(spec) = ArtPaintSeriesSpec::from_str(IDEAL_PAINT_STR) {
            for pair in [
                ("Red", RGB::RED),
                ("Green", RGB::GREEN),
                ("Blue", RGB::BLUE),
                ("Cyan", RGB::CYAN),
                ("Magenta", RGB::MAGENTA),
                ("Yellow", RGB::YELLOW),
                ("Black", RGB::BLACK),
                ("White", RGB::WHITE),
            ]
            .iter()
            {
                if let Some(index) = spec.get_index_for_name(pair.0) {
                    assert_eq!(spec.paint_specs[index].rgb, pair.1);
                } else {
                    panic!("File: {:?} Line: {:?}", file!(), line!())
                }
            }
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    //    #[test]
    //    fn art_paint_spec_obsolete_series() {
    //        match ArtPaintSeriesSpec::from_str(OBSOLETE_PAINT_STR) {
    //            Ok(spec) => {
    //                for pair in [
    //                    (
    //                        "XF 1: Flat Black *",
    //                        RGB16::from_str("RGB(0x2D00, 0x2B00, 0x3000)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 2: Flat White *",
    //                        RGB16::from_str("RGB(0xFE00, 0xFE00, 0xFE00)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 3: Flat Yellow *",
    //                        RGB16::from_str("RGB(0xF800, 0xCD00, 0x2900)").unwrap(),
    //                    ),
    //                    (
    //                        "XF 4: Yellow Green *",
    //                        RGB16::from_str("RGB(0xAA00, 0xAE00, 0x4000)").unwrap(),
    //                    ),
    //                ]
    //                .iter()
    //                {
    //                    if let Some(index) = spec.get_index_for_name(pair.0) {
    //                        assert_eq!(spec.paint_specs[index].rgb, RGB::from(pair.1));
    //                    } else {
    //                        panic!("File: {:?} Line: {:?}", file!(), line!())
    //                    }
    //                }
    //            }
    //            Err(err) => panic!("File: {:?} Line: {:?} {:?}", file!(), line!(), err),
    //        }
    //    }
}
