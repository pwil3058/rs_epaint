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
use std::path::Path;
use std::rc::Rc;

use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;
use pw_gix::gtkx::window::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use basic_paint::editor::*;
use colln_paint::*;
use colln_paint::binder::*;
use colln_paint::collection::*;
use dialogue::*;
pub use colln_paint::display::*;
use icons::series_paint_xpm::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
pub struct PaintSeriesId {
    manufacturer: String,
    series_name: String,
}

impl PaintSeriesId {
    pub fn manufacturer(&self) -> String {
        self.manufacturer.clone()
    }

    pub fn series_name(&self) -> String {
        self.series_name.clone()
    }
}

impl CollnIdInterface for PaintSeriesId {
    fn new(colln_name: &str, colln_owner: &str) -> PaintSeriesId {
        PaintSeriesId{
            manufacturer: colln_owner.to_string(),
            series_name: colln_name.to_string(),
        }
    }

    fn colln_name_label() -> String {
        "Series:".to_string()
    }

    fn colln_owner_label() -> String {
        "Manufacturer:".to_string()
    }

    fn paint_select_label() -> String {
        "Add to Mixer".to_string()
    }

    fn paint_select_tooltip_text() -> String {
        "Add this paint to the paint mixing area.".to_string()
    }

    fn recollection_name_for(item_name: &str) -> String {
        format!("series_paint::{}", item_name)
    }

    fn colln_load_image(size: i32) -> gtk::Image {
        series_paint_load_image(size)
    }

    fn colln_name(&self) -> String {
        self.series_name.clone()
    }

    fn colln_owner(&self) -> String {
        self.manufacturer.clone()
    }
}

pub type SeriesPaint<C> = CollnPaint<C, PaintSeriesId>;
pub type SeriesPaintColln<C> = CollnPaintColln<C, PaintSeriesId>;
pub type SeriesPaintCollnSpec<C> = PaintCollnSpec<C, PaintSeriesId>;

pub type SeriesPaintCollnBinder<A, C> = CollnPaintCollnBinder<A, C, PaintSeriesId>;
//pub type SeriesPaintDisplayDialog<A, C> = CollnPaintDisplayDialog<A, C, PaintSeriesId>;
pub type SeriesPaintEditor<A, C> = BasicPaintEditor<A, C, PaintSeriesId>;

pub struct SeriesPaintDisplayDialogCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: SeriesPaint<C>,
    current_target_label: gtk::Label,
    cads: Rc<A>,
    id_no: u32,
    destroyed_callbacks: RefCell<Vec<Box<Fn(u32)>>>,
}

pub type SeriesPaintDisplayDialog<A, C> = Rc<SeriesPaintDisplayDialogCore<A, C>>;

impl<A, C> DialogWrapper for SeriesPaintDisplayDialog<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn dialog(&self) -> gtk::Dialog { self.dialog. clone() }
}

impl<A, C> DialogIdentifier for SeriesPaintDisplayDialog<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn id_no(&self) -> u32 { self.id_no }
}

impl<A, C> PaintDisplayDialogCreate<A, C, SeriesPaint<C>> for SeriesPaintDisplayDialog<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        paint: &SeriesPaint<C>,
        current_target: Option<&Colour>,
        caller: &Rc<W>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> Self {
        let dialog = new_display_dialog(&paint.name(), caller, &[]);
        dialog.set_size_from_recollections("series_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let label = gtk::Label::new(paint.name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.notes().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);

        let colln_id = paint.colln_id();
        let label = gtk::Label::new(colln_id.colln_name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(colln_id.colln_owner().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        let current_target_label = gtk::Label::new("");
        current_target_label.set_widget_colour(&paint.colour());
        vbox.pack_start(&current_target_label.clone(), true, true, 0);
        //
        content_area.pack_start(&vbox, true, true, 0);
        let cads = A::create();
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
                destroyed_callbacks: RefCell::new(Vec::new()),
            }
        );
        spd_dialog.set_current_target(current_target);
        let spd_dialog_c = spd_dialog.clone();
        spd_dialog.dialog.connect_destroy(
            move |_| {
                spd_dialog_c.inform_destroyed()
            }
        );

        spd_dialog
    }

    fn paint(&self) -> SeriesPaint<C> {
        self.paint.clone()
    }

    fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if let Some(ref colour) = new_current_target {
            self.current_target_label.set_label("Current Target");
            self.current_target_label.set_widget_colour(&colour);
            self.cads.set_target_colour(Some(&colour));
        } else {
            self.current_target_label.set_label("");
            self.current_target_label.set_widget_colour(&self.paint.colour());
            self.cads.set_target_colour(None);
        };
    }

    fn connect_destroyed<F: 'static + Fn(u32)>(&self, callback: F) {
        self.destroyed_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_destroyed(&self) {
        for callback in self.destroyed_callbacks.borrow().iter() {
            callback(self.id_no);
        }
    }
}

const TOOLTIP_TEXT: &str =
"Open the Series Paint Manager.
This enables paint to be added to the mixer.";

pub struct SeriesPaintManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    window: gtk::Window,
    binder: SeriesPaintCollnBinder<A, C>,
}

impl<A,C> SeriesPaintManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    pub fn set_icon(&self, icon: &Pixbuf) {
        self.window.set_icon(Some(icon))
    }

    pub fn set_initiate_select_ok(&self, value: bool) {
        self.binder.set_initiate_select_ok(value);
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        self.binder.set_target_colour(ocolour)
    }

    pub fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.binder.connect_paint_selected(callback)
    }
}

pub type SeriesPaintManager<A, C> = Rc<SeriesPaintManagerCore<A, C>>;

pub trait SeriesPaintManagerInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C>;
    fn button(&self) -> gtk::Button;
    fn tool_button(&self) -> gtk::ToolButton;
}


impl<A, C> SeriesPaintManagerInterface<A, C> for SeriesPaintManager<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_geometry_from_recollections("series_paint_manager", (600, 200));
        window.set_destroy_with_parent(true);
        window.set_title("Series Paint Manager");
        window.connect_delete_event(
            move |w,_| {w.hide_on_delete(); gtk::Inhibit(true)}
        );
        let binder = SeriesPaintCollnBinder::<A, C>::create(data_path);
        binder.set_initiate_select_ok(true);
        window.add(&binder.pwo());

        let spm = Rc::new(
            SeriesPaintManagerCore::<A, C>{window, binder}
        );

        spm
    }

    fn button(&self) -> gtk::Button {
        let button = gtk::Button::new();
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

}
