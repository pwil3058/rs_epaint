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

use std::io::{self, Write};

use gtk;
use gtk::prelude::*;

use colour::attributes::*;
use gdkx::*;
use gtkx::coloured::*;
use paint::*;
use recollections;

pub struct SeriesPaintDisplayButtonSpec {
    pub label: String,
    pub tooltip_text: String,
    pub callback: Box<Fn()>
}

fn get_dialog_size_corrn() -> (i32, i32) {
    if let Some(corrn) = recollections::recall("dialog::size_correction") {
        if let Ok((width, height)) = parse_geometry_size(corrn.as_str()) {
            return (width, height);
        } else {
            io::stderr().write(b"Error parsing \"dialog::size_correction\"\n").unwrap();
        }
    };
    (0, 0)
}

fn recall_dialog_last_size(key: &str, default: (i32, i32)) -> (i32, i32) {
    if let Some(last_size) = recollections::recall(key) {
        if let Ok((width, height)) = parse_geometry_size(last_size.as_str()) {
            let (w_corrn, h_corrn) = get_dialog_size_corrn();
            return (width + w_corrn, height + h_corrn);
        } else {
            let msg = format!("Error parsing \"{}\"\n", key);
            io::stderr().write(msg.as_bytes()).unwrap();
        }
    };
    default
}

trait RememberDialogSize: gtk::WidgetExt + gtk::WindowExt {
    fn set_size_from_recollections(&self, dialog_name: &str, default: (i32, i32)) {
        let key = format!("{}::dialog::last_size", dialog_name);
        let (width, height) = recall_dialog_last_size(key.as_str(), default);
        self.set_default_size(width, height);
        self.connect_configure_event(
            move |_, event| {
                let text = format_geometry_size(event);
                recollections::remember(key.as_str(), text.as_str());
                false
            }
        );
        self.connect_realize(
            |widget| {
                let (req_width, req_height) = widget.get_default_size();
                let allocation = widget.get_allocation();
                let width_corrn = if req_width > 0 { req_width - allocation.width } else { 0 };
                let height_corrn = if req_height > 0 { req_height - allocation.height } else { 0 };
                let text = format!("{}x{}", width_corrn, height_corrn);
                recollections::remember("dialog::size_correction", text.as_str())
            }
        );
    }
}

impl RememberDialogSize for gtk::Dialog {}

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
        content_area.set_spacing(1);
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
        content_area.pack_start(&cads.pwo(), true, true, 0);
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
                cads: CADS::create(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
