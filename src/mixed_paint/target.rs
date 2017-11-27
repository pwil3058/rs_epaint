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

use std::cmp::*;
use std::rc::Rc;

use pw_gix::colour::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;

use paint::*;

#[derive(Debug)]
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

impl<A> TargetColourDisplayDialogCore<A>
    where   A: ColourAttributesInterface
{
    pub fn show(&self) {
        self.dialog.show()
    }
}

pub type TargetColourDisplayDialog<A> = Rc<TargetColourDisplayDialogCore<A>>;

pub trait TargetColourDisplayDialogInterface<A>
    where   A: ColourAttributesInterface
{
    fn create(
        colour: &TargetColour,
        parent: Option<&gtk::Window>,
    ) -> TargetColourDisplayDialog<A>;
}

impl<A> TargetColourDisplayDialogInterface<A> for TargetColourDisplayDialog<A>
    where   A: ColourAttributesInterface + 'static
{
    fn create(
        colour: &TargetColour,
        parent: Option<&gtk::Window>,
    ) -> TargetColourDisplayDialog<A> {
        let title = format!("mcmmtk: {}", colour.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DIALOG_USE_HEADER_BAR,
            &[]
        );
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

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
