// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cmp::*;
use std::marker::PhantomData;
use std::rc::Rc;

use pw_gix::{
    gtk::{self, prelude::*, WidgetExt},
    gtkx::dialog::*,
    wrapper::*,
};

use colour_math_gtk::coloured::*;

use crate::app_name;
use crate::basic_paint::*;
use crate::colour::*;
use crate::colour_edit::*;
use crate::dialogue::*;

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
        Rc::new(TargetColourCore {
            colour: colour.clone(),
            name: name.to_string(),
            notes: notes.to_string(),
        })
    }
}

pub struct TargetColourDisplayDialogCore<A>
where
    A: ColourAttributesInterface,
{
    dialog: gtk::Dialog,
    cads: PhantomData<A>,
}

pub type TargetColourDisplayDialog<A> = Rc<TargetColourDisplayDialogCore<A>>;

impl<A> DialogWrapper for TargetColourDisplayDialog<A>
where
    A: ColourAttributesInterface,
{
    fn dialog(&self) -> gtk::Dialog {
        self.dialog.clone()
    }
}

pub trait TargetColourDisplayDialogInterface<A>
where
    A: ColourAttributesInterface,
{
    fn create<W: WidgetWrapper>(
        colour: &TargetColour,
        caller: &Rc<W>,
    ) -> TargetColourDisplayDialog<A>;
}

impl<A> TargetColourDisplayDialogInterface<A> for TargetColourDisplayDialog<A>
where
    A: ColourAttributesInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        colour: &TargetColour,
        caller: &Rc<W>,
    ) -> TargetColourDisplayDialog<A> {
        let dialog = new_display_dialog(&colour.name(), caller, &[]);
        dialog.set_size_from_recollections("target_colour_display", (60, 180));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(Some(colour.name().as_str()));
        label.set_widget_colour(&colour.colour());
        vbox.pack_start(&label, true, true, 0);
        let label = gtk::Label::new(Some(colour.notes().as_str()));
        label.set_widget_colour(&colour.colour());
        vbox.pack_start(&label, true, true, 0);
        content_area.pack_start(&vbox, true, true, 0);
        let cads = A::create();
        cads.set_colour(Some(&colour.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        content_area.show_all();
        Rc::new(TargetColourDisplayDialogCore {
            dialog: dialog,
            cads: PhantomData,
        })
    }
}

// Entry for setting a new target colour
pub struct NewTargetColourDialogCore<A>
where
    A: ColourAttributesInterface + 'static,
{
    dialog: gtk::Dialog,
    colour_editor: ColourEditor<A>,
    notes: gtk::Entry,
}

pub type NewTargetColourDialog<A> = Rc<NewTargetColourDialogCore<A>>;

pub trait NewTargetColourDialogInterface<A>
where
    A: ColourAttributesInterface,
{
    fn create<W: WidgetWrapper>(caller: &Rc<W>) -> NewTargetColourDialog<A>;
}

impl<A> NewTargetColourDialogInterface<A> for NewTargetColourDialog<A>
where
    A: ColourAttributesInterface,
{
    fn create<W: WidgetWrapper>(caller: &Rc<W>) -> NewTargetColourDialog<A> {
        let title = format!("{}: New Mixed Paint Target Colour", app_name());
        let dialog = caller.new_dialog_with_buttons(
            Some(&title),
            gtk::DialogFlags::DESTROY_WITH_PARENT,
            CANCEL_OK_BUTTONS,
        );
        let colour_editor = ColourEditor::<A>::create(&vec![]);
        let notes = gtk::Entry::new();

        let content_area = dialog.get_content_area();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        hbox.pack_start(&gtk::Label::new(Some("Notes:")), false, false, 0);
        hbox.pack_start(&notes.clone(), true, true, 0);
        content_area.pack_start(&hbox, false, false, 0);
        content_area.pack_start(&colour_editor.pwo(), true, true, 0);
        content_area.show_all();

        let ntcd = Rc::new(NewTargetColourDialogCore::<A> {
            dialog,
            colour_editor,
            notes,
        });
        let ntcd_c = ntcd.clone();
        ntcd.notes.connect_changed(move |entry| {
            ntcd_c.dialog.set_response_sensitive(
                gtk::ResponseType::Accept.into(),
                entry.get_text().len() > 0,
            )
        });

        ntcd
    }
}

impl<A> NewTargetColourDialogCore<A>
where
    A: ColourAttributesInterface,
{
    pub fn get_new_target(&self) -> Option<(String, Colour)> {
        if gtk::ResponseType::from(self.dialog.run()) == gtk::ResponseType::Ok {
            let notes = self.notes.get_text();
            if notes.len() > 0 {
                let colour = self.colour_editor.get_colour();
                unsafe { self.dialog.destroy() };
                return Some((String::from(notes), colour));
            }
        };
        unsafe { self.dialog.destroy() };
        None
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
