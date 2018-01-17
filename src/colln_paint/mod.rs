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
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::hash::*;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::wrapper::*;

pub mod binder;
pub mod collection;
pub mod display;

use basic_paint::*;
use error::*;
pub use struct_traits::SimpleCreation;

pub trait CollnIdInterface:
    Debug + PartialEq + PartialOrd + Eq + Ord + Clone + Default + Hash
{
    fn new(colln_name: &str, colln_owner: &str) -> Self;
    fn colln_name_label() -> String;
    fn colln_owner_label() -> String;

    fn paint_select_label() -> String;
    fn paint_select_tooltip_text() -> String;

    fn recollection_name_for(item_name: &str) -> String;

    fn colln_load_image(size: i32) -> gtk::Image;

    fn display_current_target() -> bool {
        true
    }

    fn colln_name(&self) -> String;
    fn colln_owner(&self) -> String;

    fn tooltip_text(&self) -> String {
        format!("{}\n({})", self.colln_name(), self.colln_owner())
    }

    fn rc_new(colln_name: &str, colln_owner: &str) -> Rc<Self> {
        Rc::new(Self::new(colln_name, colln_owner))
    }
}

pub struct CollnIdEntryData<CID>
    where   CID: CollnIdInterface
{
    grid: gtk::Grid,
    colln_name_entry: gtk::Entry,
    colln_owner_entry: gtk::Entry,
    changed_callbacks: RefCell<Vec<Box<Fn()>>>,
    phantom_data: PhantomData<CID>,
}

impl<CID> WidgetWrapper for CollnIdEntryData<CID>
    where   CID: CollnIdInterface
{
    type PWT = gtk::Grid;

    fn pwo(&self) -> gtk::Grid {
        self.grid.clone()
    }
}

pub type CollnIdEntry<CID> = Rc<CollnIdEntryData<CID>>;

impl<CID> SimpleCreation for CollnIdEntry<CID>
    where   CID: CollnIdInterface + 'static
{
    fn create() -> CollnIdEntry<CID> {
        let psie = Rc::new(
            CollnIdEntryData {
                grid: gtk::Grid::new(),
                colln_owner_entry: gtk::Entry::new(),
                colln_name_entry: gtk::Entry::new(),
                changed_callbacks: RefCell::new(Vec::new()),
                phantom_data: PhantomData,
            }
        );
        let label = gtk::Label::new(Some(CID::colln_name_label().as_str()));
        label.set_halign(gtk::Align::End);
        psie.grid.attach(&label, 0, 0, 1, 1);
        psie.colln_name_entry.set_hexpand(true);
        psie.grid.attach_next_to(&psie.colln_name_entry.clone(), Some(&label), gtk::PositionType::Right, 1, 1);
        let label = gtk::Label::new(Some(CID::colln_owner_label().as_str()));
        label.set_halign(gtk::Align::End);
        psie.grid.attach(&label, 0, 1, 1, 1);
        psie.colln_owner_entry.set_hexpand(true);
        psie.grid.attach_next_to(&psie.colln_owner_entry.clone(), Some(&label), gtk::PositionType::Right, 1, 1);

        let psie_c = psie.clone();
        psie.colln_name_entry.connect_changed(
            move |_| psie_c.inform_changed()
        );

        let psie_c = psie.clone();
        psie.colln_owner_entry.connect_changed(
            move |_| psie_c.inform_changed()
        );

        psie
    }
}

impl<CID> CollnIdEntryData<CID>
    where   CID: CollnIdInterface
{
    pub fn get_colln_id(&self) -> Option<Rc<CID>> {
        if let Some(colln_name) = self.colln_name_entry.get_text() {
            if colln_name.len() > 0 {
                if let Some(colln_owner) = self.colln_owner_entry.get_text() {
                    if colln_owner.len() > 0 {
                        return Some(CID::rc_new(&colln_name, &colln_owner))
                    }
                }
            }
        };
        None
    }

    pub fn set_colln_id(&self, o_cid: Option<&Rc<CID>>) {
        if let Some(cid) = o_cid {
            self.colln_name_entry.set_text(&cid.colln_name());
            self.colln_owner_entry.set_text(&cid.colln_owner());
        } else {
            self.colln_name_entry.set_text("");
            self.colln_owner_entry.set_text("");
        }
    }

    pub fn connect_changed<F: 'static + Fn()>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback));
    }

    fn inform_changed(&self) {
        for callback in self.changed_callbacks.borrow().iter() {
            callback()
        }
    }
}

// COLLECTION PAINT CORE
#[derive(Debug, Hash, Clone)]
pub struct CollnPaintCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    colln_id: Rc<CID>,
    paint: BasicPaint<C>,
}

impl<C, CID> PartialEq for CollnPaintCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn eq(&self, other: &CollnPaintCore<C, CID>) -> bool {
        if self.colln_id != other.colln_id {
            false
        } else {
            self.paint == other.paint
        }
    }
}

impl<C, CID> Eq for CollnPaintCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{}

impl<C, CID> PartialOrd for CollnPaintCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn partial_cmp(&self, other: &CollnPaintCore<C, CID>) -> Option<Ordering> {
        if let Some(ordering) = self.colln_id.partial_cmp(&other.colln_id) {
            if ordering == Ordering::Equal {
                self.paint.partial_cmp(&other.paint)
            } else {
                Some(ordering)
            }
        } else {
            //panic!("File: {:?} Line: {:?}", file!(), line!())
            None
        }
    }
}

impl<C, CID> Ord for CollnPaintCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn cmp(&self, other: &CollnPaintCore<C, CID>) -> Ordering {
        let ordering = self.colln_id.cmp(&other.colln_id);
        if ordering == Ordering::Equal {
            self.paint.cmp(&other.paint)
        } else {
            ordering
        }
    }
}

// COLLECTION PAINT
pub type CollnPaint<C, CID> = Rc<CollnPaintCore<C, CID>>;

impl<C, CID> ColouredItemInterface for CollnPaint<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn colour(&self) -> Colour {
        self.paint.colour()
    }
}

impl<C, CID> BasicPaintInterface<C> for CollnPaint<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn name(&self) -> String {
        self.paint.name()
    }

    fn notes(&self) -> String {
        self.paint.notes()
    }

    fn tooltip_text(&self) -> String {
        format!("{}\n{}", self.paint.tooltip_text(), self.colln_id.tooltip_text())
    }

    fn characteristics(&self) -> C {
        self.paint.characteristics()
    }
}

pub trait CollnPaintInterface<C, CID>: BasicPaintInterface<C>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn create(paint: &BasicPaint<C>, cid: &Rc<CID>) -> Self;
    fn colln_id(&self) -> Rc<CID>;
}

impl<C, CID> CollnPaintInterface<C, CID> for CollnPaint<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn create(paint: &BasicPaint<C>, cid: &Rc<CID>) -> CollnPaint<C, CID>{
        Rc::new(
            CollnPaintCore::<C, CID> {
                colln_id: cid.clone(),
                paint: paint.clone(),
            }
        )
    }

    fn colln_id(&self) -> Rc<CID> {
        self.colln_id.clone()
    }
}

// PAINT COLLECTION SPECIFICATION
#[derive(Debug)]
pub struct PaintCollnSpec<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    pub colln_id: Rc<CID>,
    pub paint_specs: Vec<BasicPaintSpec<C>>, // sorted
}

impl<C, CID> PaintCollnSpec<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    pub fn from_file(path: &Path) -> Result<PaintCollnSpec<C, CID>, PaintError> {
        let mut file = File::open(path)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        PaintCollnSpec::<C, CID>::from_str(string.as_str())
    }

    pub fn get_index_for_name(&self, name: &str) -> Option<usize> {
        match self.paint_specs.binary_search_by_key(&name.to_string(), |spec| spec.name.clone()) {
            Ok(index) => Some(index),
            Err(_) => None
        }
    }
}

impl<C, CID> FromStr for PaintCollnSpec<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    type Err = PaintError;

    fn from_str(string: &str) -> Result<PaintCollnSpec<C, CID>, PaintError> {
        let mut lines = string.lines();
        let mut colln_name = "";
        let mut colln_owner = "";
        for _ in 0..2 {
            if let Some(line) = lines.next() {
                if line.starts_with(&CID::colln_name_label()) {
                    if let Some(tail) = line.get(CID::colln_name_label().len()..) {
                        colln_name = tail.trim();
                    }
                } else if line.starts_with(&CID::colln_owner_label()){
                    if let Some(tail) = line.get(CID::colln_owner_label().len()..) {
                        colln_owner = tail.trim();
                    }
                } else {
                    return Err(PaintErrorType::MalformedText(line.to_string()).into())
                }
            } else {
                return Err(PaintErrorType::MalformedText(string.to_string()).into())
            }
        }
        if colln_name.len() == 0 || colln_owner.len() == 0 {
            return Err(PaintErrorType::MalformedText(string.to_string()).into())
        };
        let colln_id = Rc::new(CID::new(colln_name, colln_owner));
        let mut paint_specs: Vec<BasicPaintSpec<C>> = Vec::new();
        for line in lines {
            let spec = BasicPaintSpec::<C>::from_str(line)?;
            match paint_specs.binary_search_by_key(&spec.name, |bps| bps.name.clone()) {
                Ok(_) => return Err(PaintErrorType::AlreadyExists(spec.name).into()),
                Err(index) => paint_specs.insert(index, spec)
            }
        }
        let psc = PaintCollnSpec::<C, CID>{colln_id, paint_specs};
        Ok(psc)
    }
}

impl<C, CID> fmt::Display for PaintCollnSpec<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}\n", CID::colln_name_label(), self.colln_id.colln_name())?;
        write!(f, "{} {}\n", CID::colln_owner_label(), self.colln_id.colln_owner())?;
        for paint_spec in self.paint_specs.iter() {
            write!(f, "{}\n", paint_spec)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
