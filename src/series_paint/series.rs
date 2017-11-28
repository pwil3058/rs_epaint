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
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use regex::*;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::rgb_math::rgb::*;


use display::*;
use paint::*;
use super::*;

lazy_static! {
    pub static ref MANUFACTURER_RE: Regex = Regex::new(
        r#"^Manufacturer:\s*(?P<manufacturer>.*)\s*$"#
    ).unwrap();

    pub static ref SERIES_RE: Regex = Regex::new(
        r#"^Series:\s*(?P<series>.*)\s*$"#
    ).unwrap();

    pub static ref SERIES_PAINT_RE: Regex = Regex::new(
        r#"^(?P<ptype>\w+)\((name=)?"(?P<name>.+)", rgb=(?P<rgb>RGB(16)?\([^)]+\))(?P<characteristics>(?:, \w+="\w+")*)(, notes="(?P<notes>.*)")?\)$"#
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
        let name_match = captures.name("name").ok_or(PaintError::MalformedText(string.to_string()))?;
        let characteristics = C::from_str(c_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        let rgb16 = RGB16::from_str(rgb_match.as_str()).map_err(|_| PaintError::MalformedText(string.to_string()))?;
        let notes = match captures.name("notes") {
            Some(notes_match) => notes_match.as_str().to_string(),
            None => "".to_string()
        };
        Ok(
            SeriesPaintSpec::<C> {
                rgb: RGB::from(rgb16),
                name: name_match.as_str().to_string(),
                notes: notes,
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

pub struct PaintSeriesViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    menu: gtk::Menu,
    paint_info_item: gtk::MenuItem,
    add_paint_item: gtk::MenuItem,
    series: PaintSeries<C>,
    chosen_paint: RefCell<Option<SeriesPaint<C>>>,
    current_target: RefCell<Option<Colour>>,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    series_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
    spec: PhantomData<A>
}

impl<A, C> PaintSeriesViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn get_series_paint_at(&self, posn: (f64, f64)) -> Option<SeriesPaint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self.list_store.get_value(&iter, 0).get().unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    let paint = self.series.get_series_paint(&name).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    return Some(paint)
                }
            }
        }
        None
    }

    fn inform_add_paint(&self, paint: &SeriesPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        for dialog in self.series_paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        };
        if let Some(colour) = o_colour {
            *self.current_target.borrow_mut() = Some(colour.clone())
        } else {
            *self.current_target.borrow_mut() = None
        }
    }
}

pub type PaintSeriesView<A, C> = Rc<PaintSeriesViewCore<A, C>>;

pub trait PaintSeriesViewInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow;
    fn create(series: &PaintSeries<C>) -> PaintSeriesView<A, C>;
    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F);
}

impl<A, C> PaintSeriesViewInterface<A, C> for PaintSeriesView<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }

    fn create(series: &PaintSeries<C>) -> PaintSeriesView<A, C> {
        let len = SeriesPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        for paint in series.get_series_paints().iter() {
            list_store.append_row(&paint.tv_rows());
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let menu = gtk::Menu::new();
        let paint_info_item = gtk::MenuItem::new_with_label("Information");
        menu.append(&paint_info_item.clone());
        let add_paint_item = gtk::MenuItem::new_with_label("Add to Mixer");
        add_paint_item.set_visible(false);
        menu.append(&add_paint_item.clone());
        menu.show_all();

        let mspl = Rc::new(
            PaintSeriesViewCore::<A, C> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                menu: menu,
                paint_info_item: paint_info_item.clone(),
                add_paint_item: add_paint_item.clone(),
                series: series.clone(),
                view: view,
                chosen_paint: RefCell::new(None),
                current_target: RefCell::new(None),
                add_paint_callbacks: RefCell::new(Vec::new()),
                series_paint_dialogs: RefCell::new(HashMap::new()),
                spec: PhantomData,
            }
        );

        mspl.view.append_column(&simple_text_column("Name", SP_NAME, SP_NAME, SP_RGB, SP_RGB_FG, -1, true));
        mspl.view.append_column(&simple_text_column("Notes", SP_NOTES, SP_NOTES, SP_RGB, SP_RGB_FG, -1, true));
        for col in A::tv_columns() {
            mspl.view.append_column(&col);
        }
        for col in C::tv_columns(SP_CHARS_0) {
            mspl.view.append_column(&col);
        }

        mspl.view.show_all();

        mspl.scrolled_window.add(&mspl.view.clone());
        mspl.scrolled_window.show_all();

        let mspl_c = mspl.clone();
        mspl.view.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let o_paint = mspl_c.get_series_paint_at(event.get_position());
                        mspl_c.paint_info_item.set_sensitive(o_paint.is_some());
                        mspl_c.add_paint_item.set_sensitive(o_paint.is_some());
                        let have_listeners = mspl_c.add_paint_callbacks.borrow().len() > 0;
                        mspl_c.add_paint_item.set_visible(have_listeners);
                        *mspl_c.chosen_paint.borrow_mut() = o_paint;
                        mspl_c.menu.popup_at_pointer(None);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );

        let mspl_c = mspl.clone();
        add_paint_item.connect_activate(
            move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    mspl_c.inform_add_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );

        let mspl_c = mspl.clone();
        paint_info_item.clone().connect_activate(
            move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    let target_colour = mspl_c.current_target.borrow().clone();
                    let target = if let Some(ref colour) = target_colour {
                        Some(colour)
                    } else {
                        None
                    };
                    let have_listeners = mspl_c.add_paint_callbacks.borrow().len() > 0;
                    let buttons = if have_listeners {
                        let mspl_c_c = mspl_c.clone();
                        let paint_c = paint.clone();
                        let spec = PaintDisplayButtonSpec {
                            label: "Add".to_string(),
                            tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                            callback:  Box::new(move || mspl_c_c.inform_add_paint(&paint_c))
                        };
                        vec![spec]
                    } else {
                        vec![]
                    };
                    let dialog = PaintDisplayDialog::<A, C>::series_create(
                        &paint,
                        target,
                        None,
                        buttons
                    );
                    let mspl_c_c = mspl_c.clone();
                    dialog.connect_destroy(
                        move |id| { mspl_c_c.series_paint_dialogs.borrow_mut().remove(&id); }
                    );
                    mspl_c.series_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            }
        );

        mspl
    }

    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks.borrow_mut().push(Box::new(callback))
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
        assert_eq!(captures.name("characteristics").unwrap().as_str(), ", transparency=\"O\", finish=\"F\", metallic=\"NM\", fluorescence=\"NF\"");
        assert_eq!(captures.name("notes").unwrap().as_str(), "FS37925 RAL9016 RLM21");
    }

    #[test]
    fn paint_series_paint_obsolete_regex() {
        let test_str = r#"NamedColour(name="XF 1: Flat Black *", rgb=RGB(0x2D00, 0x2B00, 0x3000), transparency="O", finish="F")"#.to_string();
        assert!(SERIES_PAINT_RE.is_match(&test_str));
        let captures = SERIES_PAINT_RE.captures(&test_str).unwrap();
        assert_eq!(captures.name("ptype").unwrap().as_str(), "NamedColour");
        assert_eq!(captures.name("rgb").unwrap().as_str(), "RGB(0x2D00, 0x2B00, 0x3000)");
        assert_eq!(captures.name("characteristics").unwrap().as_str(), ", transparency=\"O\", finish=\"F\"");
        assert_eq!(captures.name("notes"), None);
    }
}
