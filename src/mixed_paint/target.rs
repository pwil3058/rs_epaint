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

use gtk::{self, WidgetExt};
use gtk::prelude::*;

use std::cmp::*;
use std::rc::Rc;

use pw_gix::colour::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;

use app_name;
use basic_paint::*;
use colour_edit::*;
use dialogue::*;

#[derive(Debug, Hash)]
pub struct TargetColourCore {
    name: String,
    notes: String,
    colour: Colour,
}

impl TargetColourCore {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn notes(&self) -> String {
        self.notes.clone()
    }

    pub fn tooltip_text(&self) -> String {
        format!("{}: {}", self.name, self.notes)
    }
}

impl PartialEq for TargetColourCore {
    fn eq(&self, other: &TargetColourCore) -> bool {
        self.name == other.name
    }
}

impl Eq for TargetColourCore {}

impl PartialOrd for TargetColourCore {
    fn partial_cmp(&self, other: &TargetColourCore) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for TargetColourCore {
    fn cmp(&self, other: &TargetColourCore) -> Ordering {
        self.name.cmp(&other.name)
    }
}

pub type TargetColour = Rc<TargetColourCore>;

impl ColouredItemInterface for TargetColour {
    fn colour(&self) -> Colour {
        self.colour.clone()
    }
}

pub trait TargetColourInterface {
    fn create(colour: &Colour, name: &str, notes: &str) -> TargetColour;
}

impl TargetColourInterface for TargetColour {
    fn create(colour: &Colour, name: &str, notes: &str) -> TargetColour {
        Rc::new(
            TargetColourCore{
                colour: colour.clone(),
                name: name.to_string(),
                notes: notes.to_string(),
            }
        )
    }
}


pub struct TargetColourDisplayDialogCore<A>
    where   A: ColourAttributesInterface
{
    dialog: gtk::Dialog,
    cads: PhantomData<A>,
}

pub type TargetColourDisplayDialog<A> = Rc<TargetColourDisplayDialogCore<A>>;

impl<A> DialogWrapper for TargetColourDisplayDialog<A>
    where   A: ColourAttributesInterface
{
    fn dialog(&self) -> gtk::Dialog {
        self.dialog.clone()
    }
}

pub trait TargetColourDisplayDialogInterface<A>
    where   A: ColourAttributesInterface
{
    fn create<W: WidgetWrapper>(
        colour: &TargetColour,
        caller: &Rc<W>,
    ) -> TargetColourDisplayDialog<A>;
}

impl<A> TargetColourDisplayDialogInterface<A> for TargetColourDisplayDialog<A>
    where   A: ColourAttributesInterface + 'static
{
    fn create<W: WidgetWrapper>(
        colour: &TargetColour,
        caller: &Rc<W>,
    ) -> TargetColourDisplayDialog<A> {
        let dialog = new_display_dialog(&colour.name(), caller, &[]);
        dialog.set_size_from_recollections("target_colour_display", (60, 180));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(colour.name().as_str());
        label.set_widget_colour(&colour.colour());
        vbox.pack_start(&label, true, true, 0);
        let label = gtk::Label::new(colour.notes().as_str());
        label.set_widget_colour(&colour.colour());
        vbox.pack_start(&label, true, true, 0);
        content_area.pack_start(&vbox, true, true, 0);
        let cads = A::create();
        cads.set_colour(Some(&colour.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        content_area.show_all();
        Rc::new(
            TargetColourDisplayDialogCore {
                dialog: dialog,
                cads: PhantomData,
            }
        )
    }
}

// Entry for setting a new target colour
pub struct NewTargetColourDialogCore<A>
    where   A: ColourAttributesInterface + 'static
{
    dialog: gtk::Dialog,
    colour_editor: ColourEditor<A>,
    notes: gtk::Entry,
}

pub type NewTargetColourDialog<A> = Rc<NewTargetColourDialogCore<A>>;

pub trait NewTargetColourDialogInterface<A>
    where   A: ColourAttributesInterface
{
    fn create<W: WidgetWrapper>(caller: &Rc<W>) -> NewTargetColourDialog<A>;
}

impl<A> NewTargetColourDialogInterface<A> for NewTargetColourDialog<A>
    where   A: ColourAttributesInterface
{
    fn create<W: WidgetWrapper>(caller: &Rc<W>) -> NewTargetColourDialog<A> {
        let title = format!("{}: New Mixed Paint Target Colour", app_name());
        let dialog = caller.new_dialog_with_buttons(Some(&title), gtk::DialogFlags::DESTROY_WITH_PARENT, CANCEL_OK_BUTTONS);
        let colour_editor = ColourEditor::<A>::create(&vec![]);
        let notes = gtk::Entry::new();

        let content_area = dialog.get_content_area();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        hbox.pack_start(&gtk::Label::new(Some("Notes:")), false, false, 0);
        hbox.pack_start(&notes.clone(), true, true, 0);
        content_area.pack_start(&hbox, false, false, 0);
        content_area.pack_start(&colour_editor.pwo(), true, true, 0);
        content_area.show_all();

        let ntcd = Rc::new(
            NewTargetColourDialogCore::<A>{dialog, colour_editor, notes}
        );
        let ntcd_c = ntcd.clone();
        ntcd.notes.connect_changed(
            move |entry| {
                if let Some(text) = entry.get_text() {
                    ntcd_c.dialog.set_response_sensitive(gtk::ResponseType::Accept.into(), text.len() > 0)
                } else {
                    ntcd_c.dialog.set_response_sensitive(gtk::ResponseType::Accept.into(), false)
                }
            }
        );

        ntcd
    }
}

impl <A> NewTargetColourDialogCore<A>
    where   A: ColourAttributesInterface
{
    pub fn get_new_target(&self) -> Option<(String, Colour)> {
        let ok: i32 = CANCEL_OK_BUTTONS[1].1;
        if self.dialog.run() == ok {
            if let Some(notes) = self.notes.get_text() {
                let colour = self.colour_editor.get_colour();
                self.dialog.destroy();
                return Some((notes, colour));
            }
        };
        self.dialog.destroy();
        None
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
