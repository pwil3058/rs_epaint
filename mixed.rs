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

use std::marker::PhantomData;

use gtk;
use gtk::prelude::*;

use colour::attributes::*;
use gtkx::coloured::*;
use gtkx::dialog::*;
use paint::*;

pub struct MixedPaintDisplayDialogCore<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    dialog: gtk::Dialog,
    paint: PhantomData<MixedPaint<C>>,
    cads: PhantomData<CADS>,
}

impl<C, CADS> MixedPaintDisplayDialogCore<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    pub fn show(&self) {
        self.dialog.show()
    }
}

pub type MixedPaintDisplayDialog<C, CADS> = Rc<MixedPaintDisplayDialogCore<C, CADS>>;

pub trait MixedPaintDisplayDialogInterface<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    fn create(
        paint: &MixedPaint<C>,
        parent: Option<&gtk::Window>,
    ) -> MixedPaintDisplayDialog<C, CADS>;
}

impl<C, CADS> MixedPaintDisplayDialogInterface<C, CADS> for MixedPaintDisplayDialog<C, CADS>
    where   C: CharacteristicsInterface + 'static,
            CADS: ColourAttributeDisplayStackInterface + 'static
{
    fn create(
        paint: &MixedPaint<C>,
        parent: Option<&gtk::Window>,
    ) -> MixedPaintDisplayDialog<C, CADS> {
        let title = format!("mcmmtk: {}", paint.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DIALOG_USE_HEADER_BAR,
            &[]
        );
        dialog.set_size_from_recollections("mixed_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(paint.name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.notes().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        if let Some(colour) = paint.target_colour() {
            let current_target_label = gtk::Label::new("Target Colour");
            current_target_label.set_widget_colour(&colour.clone());
            vbox.pack_start(&current_target_label.clone(), true, true, 0);
        }
        //
        content_area.pack_start(&vbox, true, true, 0);
        let cads = CADS::create();
        cads.set_colour(Some(&paint.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        let characteristics_display = paint.characteristics().gui_display_widget();
        content_area.pack_start(&characteristics_display, false, false, 0);
        let label = gtk::Label::new("Component List Will Go Here");
        content_area.pack_start(&label, false, false, 0);
        content_area.show_all();
        Rc::new(
            MixedPaintDisplayDialogCore {
                dialog: dialog,
                paint: PhantomData,
                cads: PhantomData,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
