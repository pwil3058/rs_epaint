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
use pw_gix::gtkx::menu::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use display::*;
use error::*;
use paint::*;
use super::*;

lazy_static! {
    pub static ref MANUFACTURER_RE: Regex = Regex::new(
        r#"^Manufacturer:\s*(?P<manufacturer>.*)\s*$"#
    ).unwrap();

    pub static ref SERIES_RE: Regex = Regex::new(
        r#"^Series:\s*(?P<series>.*)\s*$"#
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
            let spec = BasicPaintSpec::<C>::from_str(line)?;
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

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<SeriesPaint<C>, PaintError> {
        match self.find_name(&spec.name) {
            Ok(_) => Err(PaintError::AlreadyExists(spec.name.clone())),
            Err(index) => {
                let basic_paint = BasicPaint::<C>::from_spec(spec);
                let series_paint = SeriesPaint::<C>::create(&basic_paint, &self.series_id);
                self.paints.borrow_mut().insert(index, series_paint.clone());
                Ok(series_paint)
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
    popup_menu: WrappedMenu,
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


impl<A, C> WidgetWrapper<gtk::ScrolledWindow> for PaintSeriesViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }
}

pub type PaintSeriesView<A, C> = Rc<PaintSeriesViewCore<A, C>>;

pub trait PaintSeriesViewInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(series: &PaintSeries<C>) -> PaintSeriesView<A, C>;
    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F);
}

impl<A, C> PaintSeriesViewInterface<A, C> for PaintSeriesView<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(series: &PaintSeries<C>) -> PaintSeriesView<A, C> {
        let len = SeriesPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        for paint in series.get_series_paints().iter() {
            list_store.append_row(&paint.tv_rows());
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let mspl = Rc::new(
            PaintSeriesViewCore::<A, C> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                popup_menu: WrappedMenu::new(&vec![]),
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
        mspl.popup_menu.append_item(
            "add",
            "Add to Mixer",
            "Add this paint to the mixer palette",
        ).connect_activate(
            move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    mspl_c.inform_add_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );

        let mspl_c = mspl.clone();
        mspl.popup_menu.prepend_item(
            "info",
            "Paint Information",
            "Display this paint's information",
        ).connect_activate(
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
                    dialog.set_transient_for_from(&mspl_c.pwo());
                    let mspl_c_c = mspl_c.clone();
                    dialog.connect_destroy(
                        move |id| { mspl_c_c.series_paint_dialogs.borrow_mut().remove(&id); }
                    );
                    mspl_c.series_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            }
        );

        let mspl_c = mspl.clone();
        mspl.view.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let o_paint = mspl_c.get_series_paint_at(event.get_position());
                        mspl_c.popup_menu.set_sensitivities(o_paint.is_some(), &["add", "info"]);
                        let have_listeners = mspl_c.add_paint_callbacks.borrow().len() > 0;
                        mspl_c.popup_menu.set_visibilities(have_listeners, &["add"]);
                        *mspl_c.chosen_paint.borrow_mut() = o_paint;
                        mspl_c.popup_menu.popup_at_event(event);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
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
    //use super::*;

}
