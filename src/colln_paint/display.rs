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
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;

use app_name;
use super::*;
pub use display::PaintDisplayButtonSpec;

static mut NEXT_DIALOG_ID: u32 = 0;

fn get_id_for_dialog() -> u32 {
    let id: u32;
    unsafe {
        id = NEXT_DIALOG_ID;
        NEXT_DIALOG_ID += 1;
    }
    id
}

pub struct CollnPaintDisplayDialogCore<A, C, CID>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: CollnPaint<C, CID>,
    current_target_label: gtk::Label,
    cads: Rc<A>,
    id_no: u32,
    destroy_callbacks: RefCell<Vec<Box<Fn(u32)>>>,
}

impl<A, C, CID> CollnPaintDisplayDialogCore<A, C, CID>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    pub fn set_transient_for_from<W: gtk::WidgetExt>(&self, widget: &W) {
        self.dialog.set_transient_for_from(widget)
    }

    pub fn show(&self) {
        self.dialog.show()
    }

    pub fn present(&self) {
        self.dialog.present()
    }

    pub fn close(&self) {
        self.dialog.close()
    }

    pub fn set_response_sensitive(&self, response_id: i32, setting: bool) {
        self.dialog.set_response_sensitive(response_id, setting);
    }

    pub fn paint(&self) -> CollnPaint<C, CID> {
        self.paint.clone()
    }

    pub fn id_no(&self) -> u32 {
        self.id_no
    }

    pub fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if CID::display_current_target() {
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

pub type CollnPaintDisplayDialog<A, C, CID> = Rc<CollnPaintDisplayDialogCore<A, C, CID>>;

pub trait CollnPaintDisplayDialogInterface<A, C, CID>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn create(
        paint: &CollnPaint<C, CID>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> CollnPaintDisplayDialog<A, C, CID>;

    fn create_without_target(
        paint: &CollnPaint<C, CID>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> CollnPaintDisplayDialog<A, C, CID> {
        Self::create(paint, None, parent, button_specs)
    }
}

impl<A, C, CID> CollnPaintDisplayDialogInterface<A, C, CID> for CollnPaintDisplayDialog<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn create(
        paint: &CollnPaint<C, CID>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> CollnPaintDisplayDialog<A, C, CID> {
        let title = format!("{}: {}", app_name(), paint.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DialogFlags::USE_HEADER_BAR,
            &[]
        );
        if CID::display_current_target() {
            dialog.set_size_from_recollections("colln_paint_display", (60, 330));
        } else {
            dialog.set_size_from_recollections("colln_paint_display_no_target", (60, 330));
        };
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
        if CID::display_current_target() {
            current_target_label.set_widget_colour(&paint.colour());
            vbox.pack_start(&current_target_label.clone(), true, true, 0);
        }
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
        let cpd_dialog = Rc::new(
            CollnPaintDisplayDialogCore {
                dialog: dialog,
                paint: paint.clone(),
                current_target_label: current_target_label,
                cads: cads,
                id_no: get_id_for_dialog(),
                destroy_callbacks: RefCell::new(Vec::new()),
            }
        );
        cpd_dialog.set_current_target(current_target);
        let cpd_dialog_c = cpd_dialog.clone();
        cpd_dialog.dialog.connect_destroy(
            move |_| {
                cpd_dialog_c.inform_destroy()
            }
        );

        cpd_dialog
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
