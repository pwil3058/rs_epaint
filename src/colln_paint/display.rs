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

use super::*;
pub use dialogue::*;

pub struct CollnPaintDisplayDialogCore<A, C, CID>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: CollnPaint<C, CID>,
    current_target_label: gtk::Label,
    cads: Rc<A>,
    id_no: u32,
    destroyed_callbacks: RefCell<Vec<Box<dyn Fn(u32)>>>,
}

pub type CollnPaintDisplayDialog<A, C, CID> = Rc<CollnPaintDisplayDialogCore<A, C, CID>>;

impl<A, C, CID> DialogWrapper for CollnPaintDisplayDialog<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn dialog(&self) -> gtk::Dialog {
        self.dialog.clone()
    }
}

impl<A, C, CID> TrackedDialog for CollnPaintDisplayDialog<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn id_no(&self) -> u32 {
        self.id_no
    }

    fn destroyed_callbacks(&self) -> &RefCell<Vec<Box<dyn Fn(u32)>>> {
        &self.destroyed_callbacks
    }
}

pub trait CollnPaintDisplayDialogInterface<A, C, CID>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        paint: &CollnPaint<C, CID>,
        current_target: Option<&Colour>,
        caller: &Rc<W>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> CollnPaintDisplayDialog<A, C, CID>;
}

impl<A, C, CID> PaintDisplayWithCurrentTarget<A, C, CollnPaint<C, CID>>
    for CollnPaintDisplayDialog<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        paint: &CollnPaint<C, CID>,
        current_target: Option<&Colour>,
        caller: &Rc<W>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> CollnPaintDisplayDialog<A, C, CID> {
        let dialog = new_display_dialog(&paint.name(), caller, &[]);
        if CID::display_current_target() {
            dialog.set_size_from_recollections("colln_paint_display", (60, 330));
        } else {
            dialog.set_size_from_recollections("colln_paint_display_no_target", (60, 330));
        };
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let label = gtk::Label::new(Some(paint.name().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(Some(paint.notes().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);

        let colln_id = paint.colln_id();
        let label = gtk::Label::new(Some(colln_id.colln_name().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(Some(colln_id.colln_owner().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        let current_target_label = gtk::Label::new(None);
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
            let button = dialog.add_button(
                spec.label.as_str(),
                gtk::ResponseType::Other(response_id as u16),
            );
            button.set_tooltip_text(Some(spec.tooltip_text.as_str()));
        }
        dialog.connect_response(move |_, r_id| {
            if let gtk::ResponseType::Other(r_id) = r_id {
                if (r_id as usize) < button_specs.len() {
                    (button_specs[r_id as usize].callback)()
                }
            }
        });
        let cpd_dialog = Rc::new(CollnPaintDisplayDialogCore {
            dialog: dialog,
            paint: paint.clone(),
            current_target_label: current_target_label,
            cads: cads,
            id_no: get_id_for_dialog(),
            destroyed_callbacks: DestroyedCallbacks::create(),
        });
        cpd_dialog.set_current_target(current_target);
        let cpd_dialog_c = cpd_dialog.clone();
        cpd_dialog
            .dialog
            .connect_destroy(move |_| cpd_dialog_c.inform_destroyed());

        cpd_dialog
    }

    fn paint(&self) -> CollnPaint<C, CID> {
        self.paint.clone()
    }

    fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if CID::display_current_target() {
            if let Some(ref colour) = new_current_target {
                self.current_target_label.set_label("Current Target");
                self.current_target_label.set_widget_colour(&colour);
                self.cads.set_target_colour(Some(&colour));
            } else {
                self.current_target_label.set_label("");
                self.current_target_label
                    .set_widget_colour(&self.paint.colour());
                self.cads.set_target_colour(None);
            };
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
