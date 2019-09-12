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

use basic_paint::*;
use colour_edit::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EntryStatus {
    EditingNoChange,
    EditingReady,
    EditingNotReady,
    CreatingReady,
    CreatingNotReadyNamed,
    CreatingNotReadyUnnamed,
}

impl EntryStatus {
    pub fn needs_saving(&self) -> bool {
        match *self {
            EntryStatus::EditingNoChange | EntryStatus::CreatingNotReadyUnnamed => false,
            _ => true,
        }
    }

    pub fn is_saveable(&self) -> bool {
        match *self {
            EntryStatus::EditingNoChange
            | EntryStatus::CreatingReady
            | EntryStatus::EditingReady => true,
            _ => false,
        }
    }
}

pub trait CreateInterface {
    fn create(extra_buttons: &Vec<gtk::Button>) -> Self;
}

pub struct BasicPaintSpecEntryCore<A, C>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
{
    vbox: gtk::Box,
    edited_spec: RefCell<Option<BasicPaintSpec<C>>>,
    characteristics_entry: Rc<C::Entry>,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: ColourEditor<A>,
    status_changed_callbacks: RefCell<Vec<Box<dyn Fn(EntryStatus)>>>,
}

impl_widget_wrapper!(vbox: gtk::Box, BasicPaintSpecEntryCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
);

impl<A, C> BasicPaintSpecEntryCore<A, C>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
{
    pub fn set_edited_spec(&self, o_spec: Option<BasicPaintSpec<C>>) {
        if let Some(spec) = o_spec {
            // TODO: check for unsaved changes before setting edited spec
            self.colour_editor.set_rgb(spec.rgb);
            self.name_entry.set_text(&spec.name);
            self.notes_entry.set_text(&spec.notes);
            self.characteristics_entry
                .set_characteristics(Some(&spec.characteristics));
            *self.edited_spec.borrow_mut() = Some(spec);
        } else {
            self.colour_editor.reset();
            self.name_entry.set_text("");
            self.notes_entry.set_text("");
            *self.edited_spec.borrow_mut() = None;
        };
        self.inform_status_changed();
    }

    pub fn get_basic_paint_spec(&self) -> Option<BasicPaintSpec<C>> {
        if let Some(characteristics) = self.characteristics_entry.get_characteristics() {
            if let Some(name) = self.name_entry.get_text() {
                let notes = if let Some(text) = self.notes_entry.get_text() {
                    String::from(text)
                } else {
                    "".to_string()
                };
                let spec = BasicPaintSpec::<C> {
                    rgb: self.colour_editor.get_rgb(),
                    name: String::from(name),
                    notes: notes,
                    characteristics: characteristics,
                };
                return Some(spec);
            }
        };
        None
    }

    pub fn matches_edited_spec(&self) -> bool {
        if let Some(ref edited_spec) = *self.edited_spec.borrow() {
            if let Some(ref spec) = self.get_basic_paint_spec() {
                *spec == *edited_spec
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn paint_spec_is_available(&self) -> bool {
        if self.characteristics_entry.get_characteristics().is_some() {
            self.name_entry.get_text_length() > 0
        } else {
            false
        }
    }

    pub fn get_status(&self) -> EntryStatus {
        if let Some(ref edited_spec) = *self.edited_spec.borrow() {
            if let Some(ref spec) = self.get_basic_paint_spec() {
                if *spec == *edited_spec {
                    EntryStatus::EditingNoChange
                } else {
                    EntryStatus::EditingReady
                }
            } else {
                EntryStatus::EditingNotReady
            }
        } else if self.name_entry.get_text_length() > 0 {
            if self.characteristics_entry.get_characteristics().is_some() {
                EntryStatus::CreatingReady
            } else {
                EntryStatus::CreatingNotReadyNamed
            }
        } else {
            EntryStatus::CreatingNotReadyUnnamed
        }
    }

    pub fn connect_status_changed<F: 'static + Fn(EntryStatus)>(&self, callback: F) {
        self.status_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_status_changed(&self) {
        let status = self.get_status();
        for callback in self.status_changed_callbacks.borrow().iter() {
            callback(status);
        }
    }
}

impl<A, C> CreateInterface for Rc<BasicPaintSpecEntryCore<A, C>>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
{
    fn create(extra_buttons: &Vec<gtk::Button>) -> Self {
        let spe = Rc::new(BasicPaintSpecEntryCore::<A, C> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            edited_spec: RefCell::new(None),
            characteristics_entry: C::Entry::create(),
            name_entry: gtk::Entry::new(),
            notes_entry: gtk::Entry::new(),
            colour_editor: ColourEditor::<A>::create(extra_buttons),
            status_changed_callbacks: RefCell::new(Vec::new()),
        });

        spe.name_entry.set_hexpand(true);
        spe.notes_entry.set_hexpand(true);

        let grid = spe.characteristics_entry.pwo();
        grid.insert_row(0);
        grid.insert_row(0);
        let label = gtk::Label::new(Some("Name:"));
        label.set_halign(gtk::Align::End);
        grid.attach(&label, 0, 0, 1, 1);
        grid.attach_next_to(
            &spe.name_entry.clone(),
            Some(&label),
            gtk::PositionType::Right,
            1,
            1,
        );
        let label = gtk::Label::new(Some("Notes:"));
        label.set_halign(gtk::Align::End);
        grid.attach(&label, 0, 1, 1, 1);
        grid.attach_next_to(
            &spe.notes_entry.clone(),
            Some(&label),
            gtk::PositionType::Right,
            1,
            1,
        );

        spe.vbox.pack_start(&grid, false, false, 0);
        spe.vbox
            .pack_start(&spe.colour_editor.pwo(), false, false, 0);

        spe.vbox.show_all();

        let spe_c = spe.clone();
        spe.name_entry
            .connect_changed(move |_| spe_c.inform_status_changed());

        let spe_c = spe.clone();
        spe.characteristics_entry
            .connect_changed(move || spe_c.inform_status_changed());

        let spe_c = spe.clone();
        spe.colour_editor
            .connect_colour_changed(move |_| spe_c.inform_status_changed());

        spe
    }
}

pub type BasicPaintSpecEntry<A, C> = Rc<BasicPaintSpecEntryCore<A, C>>;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
