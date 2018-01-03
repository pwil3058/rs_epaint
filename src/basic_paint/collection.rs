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
use std::cmp::{Ordering};
use std::marker::PhantomData;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::tree_view_column::*;

use struct_traits::SimpleCreation;

use basic_paint::*;

pub trait CollnIdInterface:
    Debug + PartialEq + PartialOrd + Eq + Ord + Clone + Default + Hash
{
    fn new(colln_name: &str, colln_owner: &str) -> Self;
    fn colln_name_label() -> String;
    fn colln_owner_label() -> String;

    fn colln_name(&self) -> String;
    fn colln_owner(&self) -> String;

    fn tooltip_text(&self) -> String {
        format!("{}\n({})", self.colln_name(), self.colln_owner())
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

impl<CID> WidgetWrapper<gtk::Grid> for CollnIdEntryData<CID>
    where   CID: CollnIdInterface
{
    fn pwo(&self) -> gtk::Grid {
        self.grid.clone()
    }
}

pub type CollnIdEntry<CID> = Rc<CollnIdEntryData<CID>>;

impl<CID> SimpleCreation for CollnIdEntry<CID>
    where   CID: CollnIdInterface
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

        psie
    }
}

impl<CID> CollnIdEntryData<CID>
    where   CID: CollnIdInterface
{
    pub fn get_colln_id(&self) -> Option<CID> {
        if let Some(colln_name) = self.colln_name_entry.get_text() {
            if let Some(colln_owner) = self.colln_name_entry.get_text() {
                return Some(CID::new(&colln_name, &colln_owner))
            }
        };
        None
    }

    pub fn set_colln_id(&self, o_cid: Option<&CID>) {
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
    fn from(paint: &BasicPaint<C>, cid: &Rc<CID>) -> Self;
    fn colln_id(&self) -> Rc<CID>;
}

impl<C, CID> CollnPaintInterface<C, CID> for CollnPaint<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface
{
    fn from(paint: &BasicPaint<C>, cid: &Rc<CID>) -> CollnPaint<C, CID>{
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

// COLLECTION
pub struct CollnPaintCollnCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface,
{
    colln_id: Rc<CID>,
    paints: RefCell<Vec<CollnPaint<C, CID>>>,
}

impl<C, CID> CollnPaintCollnCore<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface,
{
    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints.borrow().binary_search_by_key(
            &name.to_string(),
            |paint| paint.name()
        )
    }

    pub fn colln_id(&self) -> Rc<CID> {
        self.colln_id.clone()
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<CollnPaint<C, CID>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn get_paints(&self) -> Vec<CollnPaint<C, CID>> {
        let mut v: Vec<CollnPaint<C, CID>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.clone())
        };
        v
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }
}

pub type CollnPaintColln<C, CID> = Rc<CollnPaintCollnCore<C, CID>>;

pub trait CollnPaintCollnInterface<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface,
{
    fn create(cid: CID) -> CollnPaintColln<C, CID>;
}


impl<C, CID> CollnPaintCollnInterface<C, CID> for CollnPaintColln<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface,
{
    fn create(cid: CID) -> CollnPaintColln<C, CID> {
        let colln_id = Rc::new(cid);
        let paints: RefCell<Vec<CollnPaint<C, CID>>> = RefCell::new(Vec::new());
        Rc::new(CollnPaintCollnCore::<C, CID>{colln_id, paints})
    }
}

pub struct CollnPaintCollnViewCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    colln: CollnPaintColln<C, CID>,
    chosen_paint: RefCell<Option<CollnPaint<C, CID>>>,
    phantom_data: PhantomData<A>
}

impl<A, C, CID> CollnPaintCollnViewCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    pub fn set_chosen_paint_from_position(&self, posn: (f64, f64)) -> Option<CollnPaint<C, CID>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self.list_store.get_value(&iter, 0).get().unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    let paint = self.colln.get_paint(&name).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    *self.chosen_paint.borrow_mut() = Some(paint.clone());
                    return Some(paint)
                }
            }
        };
        *self.chosen_paint.borrow_mut() = None;
        None
    }

    pub fn colln_id(&self) -> Rc<CID> {
        self.colln.colln_id()
    }

    pub fn len(&self) -> usize {
        self.colln.len()
    }

    pub fn get_paint(&self, name: &str) -> Option<CollnPaint<C, CID>> {
        self.colln.get_paint(name)
    }

    pub fn get_paints(&self) -> Vec<CollnPaint<C, CID>> {
        self.colln.get_paints()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.colln.has_paint_named(name)
    }
}

impl<A, C, CID> WidgetWrapper<gtk::ScrolledWindow> for CollnPaintCollnViewCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }
}

pub type CollnPaintCollnView<A, C, CID> = Rc<CollnPaintCollnViewCore<A, C, CID>>;

pub trait CollnPaintCollnViewInterface<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    fn create(colln: &CollnPaintColln<C, CID>) -> CollnPaintCollnView<A, C, CID>;
}

impl<A, C, CID> CollnPaintCollnViewInterface<A, C, CID> for CollnPaintCollnView<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    fn create(colln: &CollnPaintColln<C, CID>) -> CollnPaintCollnView<A, C, CID> {
        let len = CollnPaint::<C, CID>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        for paint in colln.get_paints().iter() {
            list_store.append_row(&paint.tv_rows());
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let mspl = Rc::new(
            CollnPaintCollnViewCore::<A, C, CID> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                colln: colln.clone(),
                view: view,
                chosen_paint: RefCell::new(None),
                phantom_data: PhantomData,
            }
        );

        mspl.view.append_column(&simple_text_column("Name", SP_NAME, SP_NAME, SP_RGB, SP_RGB_FG, -1, true));
        mspl.view.append_column(&simple_text_column("Notes", SP_NOTES, SP_NOTES, SP_RGB, SP_RGB_FG, -1, true));
        for col in A::tv_columns() {
            mspl.view.append_column(&col);
        }
        for col in C::tv_columns(SP_CHARS_0) {
            mspl.view.append_column(&col);
        }

        mspl.view.show_all();

        mspl.scrolled_window.add(&mspl.view.clone());
        mspl.scrolled_window.show_all();

        mspl
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
