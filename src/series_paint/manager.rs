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
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::dialogue::*;
use pw_gix::gtkx::notebook::*;
use pw_gix::gtkx::paned::*;
use pw_gix::gtkx::window::*;
use pw_gix::pwo::*;

use icons::series_paint_xpm::*;

use super::*;
use super::hue_wheel::*;

pub struct PaintSelectorCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    vbox: gtk::Box,
    hue_attr_wheels: Vec<SeriesPaintHueAttrWheel<A, C>>,
    paint_list: PaintSeriesView<A, C>,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
}

pub type PaintSelector<A, C> = Rc<PaintSelectorCore<A, C>>;

pub trait PaintSelectorInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::Box;
    fn create(series: &PaintSeries<C>) -> PaintSelector<A, C>;
    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F);
    fn set_target_colour(&self, ocolour: Option<&Colour>);
}

impl<A, C> PaintSelectorCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn inform_add_paint(&self, paint: &SeriesPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }
}

impl<A, C> PaintSelectorInterface<A, C> for PaintSelector<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }

    fn create(series: &PaintSeries<C>) -> PaintSelector<A, C> {
        let mut view_attr_wheels:Vec<SeriesPaintHueAttrWheel<A, C>> = Vec::new();
        for attr in A::scalar_attributes().iter() {
            view_attr_wheels.push(SeriesPaintHueAttrWheel::<A, C>::create(*attr));
        }
        let paint_selector = Rc::new(
            PaintSelectorCore::<A, C> {
                vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
                hue_attr_wheels: view_attr_wheels,
                paint_list: PaintSeriesView::<A, C>::create(series),
                add_paint_callbacks: RefCell::new(Vec::new()),
            }
        );
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let series_name = format!("Series Name: {}", series.series_id().series_name());
        hbox.pack_start(&gtk::Label::new(Some(series_name.as_str())), true, true, 0);
        let series_name = format!("Manufacturer: {}", series.series_id().manufacturer());
        hbox.pack_start(&gtk::Label::new(Some(series_name.as_str())), true, true, 0);

        let notebook = gtk::Notebook::new();
        for wheel in paint_selector.hue_attr_wheels.iter() {
            let label_text = format!("Hue/{} Wheel", wheel.attr().to_string());
            let label = gtk::Label::new(Some(label_text.as_str()));
            notebook.append_page(&wheel.pwo(), Some(&label));
        }
        let hpaned = gtk::Paned::new(gtk::Orientation::Horizontal);
        hpaned.pack1(&notebook, true, true);
        hpaned.pack2(&paint_selector.paint_list.pwo() , true, true);
        hpaned.set_position_from_recollections("model_paint_selector", 200);
        paint_selector.vbox.pack_start(&hpaned, true, true, 0);

        for paint in series.get_series_paints().iter() {
            for wheel in paint_selector.hue_attr_wheels.iter() {
                wheel.add_paint(&paint);
            }
        }

        for wheel in paint_selector.hue_attr_wheels.iter() {
            let paint_selector_c = paint_selector.clone();
            wheel.connect_add_paint(
                move |paint| paint_selector_c.inform_add_paint(paint)
            );
        }
        let paint_selector_c = paint_selector.clone();
        paint_selector.paint_list.connect_add_paint(
            move |paint| paint_selector_c.inform_add_paint(paint)
        );

        paint_selector
    }

    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn set_target_colour(&self, ocolour: Option<&Colour>) {
        for wheel in self.hue_attr_wheels.iter() {
            wheel.set_target_colour(ocolour);
        }
        self.paint_list.set_target_colour(ocolour);
    }
}

pub struct SeriesPaintManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    window: gtk::Window,
    notebook: gtk::Notebook,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    paint_selectors: RefCell<HashMap<PaintSeriesIdentity, PaintSelector<A, C>>>,
    paint_series_files_data_path: PathBuf,
}

impl<A, C> SeriesPaintManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn inform_add_paint(&self, paint: &SeriesPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    fn remove_paint_series(&self, ps_id: &PaintSeriesIdentity) {
        let mut selectors = self.paint_selectors.borrow_mut();
        if let Some(selector) = selectors.remove(ps_id) {
            let page_num = self.notebook.page_num(&selector.pwo());
            self.notebook.remove_page(page_num);
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        for selector in self.paint_selectors.borrow().values() {
            selector.set_target_colour(ocolour);
        }
    }

    pub fn get_series_file_paths(&self) -> Vec<PathBuf> {
        let mut vpb = Vec::new();
        if !self.paint_series_files_data_path.exists() {
            return vpb
        };
        let mut file = File::open(&self.paint_series_files_data_path).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        for line in string.lines() {
            vpb.push(PathBuf::from(line));
        }

        vpb
    }

    pub fn set_series_file_paths(&self, file_paths: &Vec<PathBuf>) {
        let mut file = File::create(&self.paint_series_files_data_path).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        for file_path in file_paths.iter() {
            if let Some(file_path_str) = file_path.to_str() {
                write!(file, "{}\n", file_path_str).unwrap_or_else(
                    |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
                );
            } else  {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            };
        }
}
}

pub type SeriesPaintManager<A, C> = Rc<SeriesPaintManagerCore<A, C>>;

pub trait SeriesPaintManagerInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C>;
    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F);
    fn add_paint_series(&self, series: &PaintSeries<C>);
    fn button(&self) -> gtk::Button;
    fn tool_button(&self) -> gtk::ToolButton;
}

const TOOLTIP_TEXT: &str =
"Open the Series Paint Manager.
This enables paint to be added to the mixer.";

impl<A, C> SeriesPaintManagerInterface<A, C> for SeriesPaintManager<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_geometry_from_recollections("series_paint_manager", (600, 200));
        window.set_destroy_with_parent(true);
        window.set_title("mcmmtk: Series Paint Manager");
        window.connect_delete_event(
            move |w,_| {w.hide_on_delete(); gtk::Inhibit(true)}
        );
        let notebook = gtk:: Notebook::new();
        notebook.set_scrollable(true);
        notebook.popup_enable();
        window.add(&notebook.clone());
        let add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>> = RefCell::new(Vec::new());
        let paint_selectors: RefCell<HashMap<PaintSeriesIdentity, PaintSelector<A, C>>> = RefCell::new(HashMap::new());

        let spm = Rc::new(
            SeriesPaintManagerCore::<A, C>{
                window: window,
                notebook: notebook,
                add_paint_callbacks: add_paint_callbacks,
                paint_selectors: paint_selectors,
                paint_series_files_data_path: data_path.to_path_buf(),
            }
        );
        let series_file_paths = spm.get_series_file_paths();
        for series_file_path in series_file_paths.iter() {
            if let Ok(series) = PaintSeries::<C>::from_file(&series_file_path) {
                spm.add_paint_series(&series);
            } else {
                let expln = format!("Error parsing \"{:?}\"\n", series_file_path);
                let msg = "Malformed Paint Series Text";
                warn_user(Some(&spm.window), msg, Some(expln.as_str()));
            }
        };
        spm.notebook.show_all();

        spm
    }

    fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn add_paint_series(&self, series: &PaintSeries<C>) {
        let mut selectors = self.paint_selectors.borrow_mut();
        if selectors.contains_key(&series.series_id()) {
            let expln = format!("{} ({}): already included in the tool box.\nSkipped.", series.series_id().series_name(), series.series_id().manufacturer());
            inform_user(Some(&self.window), "Duplicate Paint Series", Some(expln.as_str()));
            return;
        }
        let paint_selector = PaintSelector::<A, C>::create(&series);
        selectors.insert(series.series_id(), paint_selector.clone());
        let spm_c = self.clone();
        paint_selector.connect_add_paint(
            move |paint| spm_c.inform_add_paint(paint)
        );
        let l_text = format!("{}\n{}", series.series_id().series_name(), series.series_id().manufacturer());
        let tt_text = format!("Remove {} ({}) from the tool kit", series.series_id().series_name(), series.series_id().manufacturer());
        let label = TabRemoveLabel::create(Some(l_text.as_str()), Some(&tt_text.as_str()));
        let l_text = format!("{} ({})", series.series_id().series_name(), series.series_id().manufacturer());
        let menu_label = gtk::Label::new(Some(l_text.as_str()));
        let spm_c = self.clone();
        let ps_id = series.series_id();
        label.connect_remove_page(
            move || spm_c.remove_paint_series(&ps_id)
        );
        self.notebook.append_page_menu(&paint_selector.pwo(), Some(&label.pwo()), Some(&menu_label));
    }

    fn button(&self) -> gtk::Button {
        let button = gtk::Button::new(); //_with_label("Series Paint Manager");
        button.set_tooltip_text(Some(TOOLTIP_TEXT));
        button.set_image(&series_paint_image(24));
        let spm_c = self.clone();
        button.connect_clicked(
            move |_| spm_c.window.present()
        );
        button
    }

    fn tool_button(&self) -> gtk::ToolButton {
        let tool_button = gtk::ToolButton::new(Some(&series_paint_image(24)), Some("Series Paint Manager"));
        tool_button.set_tooltip_text(Some(TOOLTIP_TEXT));
        let spm_c = self.clone();
        tool_button.connect_clicked(
            move |_| spm_c.window.present()
        );
        tool_button
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
