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

use basic_paint::*;
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

pub struct BasicPaintDisplayDialogCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: BasicPaint<C>,
    _cads: Rc<A>,
    id_no: u32,
    destroy_callbacks: RefCell<Vec<Box<Fn(u32)>>>,
}

impl<A, C> BasicPaintDisplayDialogCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    pub fn set_transient_for_from<W: gtk::WidgetExt>(&self, widget: &W) {
        self.dialog.set_transient_for_from(widget)
    }

    pub fn show(&self) {
        self.dialog.show()
    }

    pub fn close(&self) {
        self.dialog.close()
    }

    pub fn set_response_sensitive(&self, response_id: i32, setting: bool) {
        self.dialog.set_response_sensitive(response_id, setting);
    }

    pub fn paint(&self) -> BasicPaint<C> {
        self.paint.clone()
    }

    pub fn id_no(&self) -> u32 {
        self.id_no
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

pub type BasicPaintDisplayDialog<A, C> = Rc<BasicPaintDisplayDialogCore<A, C>>;

pub trait BasicPaintDisplayDialogInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn create(
        paint: &BasicPaint<C>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> BasicPaintDisplayDialog<A, C>;
}

impl<A, C> BasicPaintDisplayDialogInterface<A, C> for BasicPaintDisplayDialog<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(
        paint: &BasicPaint<C>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> BasicPaintDisplayDialog<A, C> {
        let title = format!("mcmmtk: {}", paint.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DialogFlags::USE_HEADER_BAR,
            &[]
        );
        dialog.set_size_from_recollections("basic_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(paint.name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.notes().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
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
        let bpd_dialog = Rc::new(
            BasicPaintDisplayDialogCore {
                dialog: dialog,
                paint: paint.clone(),
                _cads: cads,
                id_no: get_id_for_dialog(),
                destroy_callbacks: RefCell::new(Vec::new()),
            }
        );
        let bpd_dialog_c = bpd_dialog.clone();
        bpd_dialog.dialog.connect_destroy(
            move |_| {
                bpd_dialog_c.inform_destroy()
            }
        );

        bpd_dialog
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
