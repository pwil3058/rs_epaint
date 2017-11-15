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

use colour_edit::*;
use series_paint::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SeriesPaintEntryStatus {
    EditingPaintReady,
    EditingPaintNotReady,
    CreatingPaintReady,
    CreatingPaintNotReady,
}

pub trait SeriesPaintEntryInterface<C>
    where C: CharacteristicsInterface
{
    fn create() -> Self;
    fn pwo(&self) -> gtk::Box;
}

pub struct SeriesPaintEntryCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    vbox: gtk::Box,
    edited_paint: RefCell<Option<SeriesPaint<C>>>,
    characteristics_entry: C::Entry,
    name_entry: gtk::Entry,
    notes_entry: gtk::Entry,
    colour_editor: ColourEditor<A>,
    status_changed_callbacks: RefCell<Vec<Box<Fn(SeriesPaintEntryStatus)>>>,
}

impl<A, C> SeriesPaintEntryCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    pub fn get_edited_paint(&self) -> Option<SeriesPaint<C>> {
        if let Some(ref paint) = *self.edited_paint.borrow() {
            Some(paint.clone())
        } else {
            None
        }
    }

    pub fn set_edited_paint(&self, o_paint: Option<&SeriesPaint<C>>) {
        if let Some(paint) = o_paint {
            // TODO: check for unsaved changes before setting edited paint
            *self.edited_paint.borrow_mut() = Some(paint.clone());
            self.colour_editor.set_rgb(paint.colour().rgb());
            self.name_entry.set_text(&paint.name());
            self.notes_entry.set_text(&paint.notes());
            self.characteristics_entry.set_characteristics(Some(&paint.characteristics()));
        } else {
            *self.edited_paint.borrow_mut() = None;
        };
        self.inform_status_changed();
    }

    //fn get_paint_specification(&self) -> Option<SeriesPaintSpec<C>>;

    pub fn get_status(&self) -> SeriesPaintEntryStatus {
        if self.characteristics_entry.get_characteristics().is_some() {
            if self.name_entry.get_text_length() == 0 {
                if self.edited_paint.borrow().is_some() {
                    SeriesPaintEntryStatus::EditingPaintNotReady
                } else {
                    SeriesPaintEntryStatus::CreatingPaintNotReady
                }
            } else if self.edited_paint.borrow().is_some() {
                    SeriesPaintEntryStatus::EditingPaintReady
            } else {
                    SeriesPaintEntryStatus::CreatingPaintReady
            }
        } else if self.edited_paint.borrow().is_some() {
            SeriesPaintEntryStatus::EditingPaintNotReady
        } else {
            SeriesPaintEntryStatus::CreatingPaintNotReady
        }
    }

    //fn connect_status_changed<F: 'static + Fn(SeriesPaintEntryStatus)>(&self, callback: F);

    fn inform_status_changed(&self) {
        let status = self.get_status();
        for callback in self.status_changed_callbacks.borrow().iter() {
            callback(status);
        }
    }
}

impl<A, C> SeriesPaintEntryInterface<C> for  Rc<SeriesPaintEntryCore<A, C>>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }

    fn create() -> Self {
        let spe = Rc::new(
            SeriesPaintEntryCore::<A, C>{
                vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
                edited_paint: RefCell::new(None),
                characteristics_entry: C::Entry::create(),
                name_entry: gtk::Entry::new(),
                notes_entry: gtk::Entry::new(),
                colour_editor: ColourEditor::<A>::create(&vec![]),
                status_changed_callbacks: RefCell::new(Vec::new()),
            }
        );

        spe.name_entry.set_hexpand(true);
        spe.notes_entry.set_hexpand(true);

        let grid = spe.characteristics_entry.pwo();
        grid.insert_row(0); grid.insert_row(0);
        let label = gtk::Label::new(Some("Name:"));
        label.set_halign(gtk::Align::End);
        grid.attach(&label, 0, 0, 1, 1);
        grid.attach_next_to(&spe.name_entry.clone(), Some(&label), gtk::PositionType::Right, 1, 1);
        let label = gtk::Label::new(Some("Notes:"));
        label.set_halign(gtk::Align::End);
        grid.attach(&label, 0, 1, 1, 1);
        grid.attach_next_to(&spe.notes_entry.clone(), Some(&label), gtk::PositionType::Right, 1, 1);

        spe.vbox.pack_start(&grid, false, false, 0);
        spe.vbox.pack_start(&spe.colour_editor.pwo(), false, false, 0);

        spe.vbox.show_all();

        let spe_c = spe.clone();
        spe.name_entry.connect_changed(
            move |_| spe_c.inform_status_changed()
        );

        let spe_c = spe.clone();
        spe.characteristics_entry.connect_changed(
            move || spe_c.inform_status_changed()
        );

        spe
    }
}

pub type SeriesPaintEntry<A, C> = Rc<SeriesPaintEntryCore<A, C>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
