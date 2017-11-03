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

use std::fs::File;
use std::io::Read;
use std::path::Path;

use gtk;
use gtk::prelude::*;

use colour::attributes::*;
use gtkx::coloured::*;
use gtkx::dialog::*;
use paint::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
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

impl<C> SeriesPaintInterface<C> for SeriesPaint<C>
    where   C: CharacteristicsInterface
{
    fn series(&self) -> PaintSeriesIdentity {
        self.series_id.clone()
    }
}

pub struct SeriesPaintDisplayButtonSpec {
    pub label: String,
    pub tooltip_text: String,
    pub callback: Box<Fn()>
}

static mut NEXT_DIALOG_ID: u32 = 0;

fn get_id_for_dialog() -> u32 {
    let id: u32;
    unsafe {
        id = NEXT_DIALOG_ID;
        NEXT_DIALOG_ID += 1;
    }
    id
}

pub struct SeriesPaintDisplayDialogCore<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    dialog: gtk::Dialog,
    paint: SeriesPaint<C>,
    current_target_label: gtk::Label,
    cads: CADS,
    id_no: u32,
    destroy_callbacks: RefCell<Vec<Box<Fn(u32)>>>
}

impl<C, CADS> SeriesPaintDisplayDialogCore<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    pub fn show(&self) {
        self.dialog.show()
    }

    pub fn id_no(&self) -> u32 {
        self.id_no
    }

    pub fn set_current_target(&self, new_current_target: Option<Colour>) {
        if let Some(colour) = new_current_target {
            self.current_target_label.set_label("Current Target");
            self.current_target_label.set_widget_colour(&colour);
            self.cads.set_target_colour(Some(&colour));
        } else {
            self.current_target_label.set_label("");
            self.current_target_label.set_widget_colour(&self.paint.colour());
            self.cads.set_target_colour(None);
        };
    }

    pub fn connect_destroy<F: 'static + Fn(u32)>(&self, callback: F) {
        self.destroy_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_destroy(&self) {
        for callback in self.destroy_callbacks.borrow().iter() {
            callback(self.id_no);
        }
    }

}

pub type SeriesPaintDisplayDialog<C, CADS> = Rc<SeriesPaintDisplayDialogCore<C, CADS>>;

pub trait SeriesPaintDisplayDialogInterface<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    fn create(
        paint: &SeriesPaint<C>,
        current_target: Option<Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<SeriesPaintDisplayButtonSpec>,
    ) -> SeriesPaintDisplayDialog<C, CADS>;
}

impl<C, CADS> SeriesPaintDisplayDialogInterface<C, CADS> for SeriesPaintDisplayDialog<C, CADS>
    where   C: CharacteristicsInterface + 'static,
            CADS: ColourAttributeDisplayStackInterface + 'static
{
    fn create(
        paint: &SeriesPaint<C>,
        current_target: Option<Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<SeriesPaintDisplayButtonSpec>,
    ) -> SeriesPaintDisplayDialog<C, CADS> {
        let title = format!("mcmmtk: {}", paint.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DIALOG_USE_HEADER_BAR,
            &[]
        );
        dialog.set_size_from_recollections("series_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(paint.name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.notes().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.series().series_name.as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.series().manufacturer.as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        let current_target_label = gtk::Label::new("");
        current_target_label.set_widget_colour(&paint.colour());
        vbox.pack_start(&current_target_label.clone(), true, true, 0);
        //
        content_area.pack_start(&vbox, true, true, 0);
        let cads = CADS::create();
        cads.set_colour(Some(&paint.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        let characteristics_display = paint.characteristics().gui_display_widget();
        content_area.pack_start(&characteristics_display, false, false, 0);
        content_area.show_all();
        for (response_id, spec) in button_specs.iter().enumerate() {
            let button = dialog.add_button(spec.label.as_str(), response_id as i32);
            button.set_tooltip_text(Some(spec.tooltip_text.as_str()));
        };
        dialog.connect_response (
            move |_, r_id| {
                if r_id >= 0 && r_id < button_specs.len() as i32 {
                    (button_specs[r_id as usize].callback)()
                }
            }
        );
        let spd_dialog = Rc::new(
            SeriesPaintDisplayDialogCore {
                dialog: dialog,
                paint: paint.clone(),
                current_target_label: current_target_label,
                cads: cads,
                id_no: get_id_for_dialog(),
                destroy_callbacks: RefCell::new(Vec::new()),
            }
        );
        spd_dialog.set_current_target(current_target);
        let spd_dialog_c = spd_dialog.clone();
        spd_dialog.dialog.connect_destroy(
            move |_| {
                spd_dialog_c.inform_destroy()
            }
        );

        spd_dialog
    }
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
    pub rgb: RGB,
    pub name: String,
    pub notes: String,
    pub characteristics: C,
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

    pub fn get_paints(&self) -> Vec<Paint<C>> {
        let mut v: Vec<Paint<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(Paint::Series(paint.clone()))
        };
        v
    }

    pub fn get_series_paint(&self, name: &str) -> Option<SeriesPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn get_series_paints(&self) -> Vec<SeriesPaint<C>> {
        let mut v: Vec<SeriesPaint<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.clone())
        };
        v
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

    fn from_file(path: &Path) -> Result<PaintSeries<C>, PaintError> {
        let mut file = File::open(path).map_err(
            |err| PaintError::IOError(err)
        )?;
        let mut string = String::new();
        file.read_to_string(&mut string).map_err(
            |err| PaintError::IOError(err)
        )?;
        PaintSeries::<C>::from_str(string.as_str())
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
