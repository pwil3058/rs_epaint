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
use std::marker::PhantomData;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::tree_view_column::*;

use basic_paint::*;

// COLLECTION PAINT
pub trait CollectionIdEntryInterface<CID: CollectionIdInterface> {
    fn create() -> Self;
    fn pwo(&self) -> gtk::Grid;
    fn get_colln_id(&self) -> Option<CID>;
    fn set_colln_id(&self, o_characteristics: Option<&CID>);
    fn connect_changed<F: 'static + Fn()>(&self, callback: F);
}

pub trait CollectionIdInterface: Debug + PartialEq + Clone + Copy {
    type Entry: CollectionIdEntryInterface<Self>;

    fn create() -> Self;
}

pub trait CollnPaintInterface<C, CID>: BasicPaintInterface<C>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface
{
    fn create(cid: &Rc<CID>, spec: &BasicPaintSpec<C>) -> Self;
    fn colln_id(&self) -> Rc<CID>;
}

pub struct CollnPaintCollnCore<C, P, CID>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>,
{
    colln_id: Rc<CID>,
    paints: RefCell<Vec<P>>,
    phantom: PhantomData<C>
}

impl<C, P, CID> CollnPaintCollnCore<C, P, CID>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>,
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

    pub fn get_paint(&self, name: &str) -> Option<P> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn get_paints(&self) -> Vec<P> {
        let mut v: Vec<P> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.clone())
        };
        v
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }
}

pub type CollnPaintColln<C, P, CID> = Rc<CollnPaintCollnCore<C, P, CID>>;

pub trait CollnPaintCollnInterface<C, P, CID>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    fn create(cid: CID) -> CollnPaintColln<C, P, CID>;
}


impl<C, P, CID> CollnPaintCollnInterface<C, P, CID> for CollnPaintColln<C, P, CID>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    fn create(cid: CID) -> CollnPaintColln<C, P, CID> {
        let colln_id = Rc::new(cid);
        let paints: RefCell<Vec<P>> = RefCell::new(Vec::new());
        let phantom = PhantomData;
        Rc::new(CollnPaintCollnCore::<C, P, CID>{colln_id, paints, phantom})
    }
}

pub struct CollnPaintCollnViewCore<A, C, P, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    colln: CollnPaintColln<C, P, CID>,
    chosen_paint: RefCell<Option<P>>,
    spec: PhantomData<A>
}

impl<A, C, P, CID> CollnPaintCollnViewCore<A, C, P, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    pub fn set_chosen_paint_from_position(&self, posn: (f64, f64)) -> Option<P> {
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

    pub fn get_paint(&self, name: &str) -> Option<P> {
        self.colln.get_paint(name)
    }

    pub fn get_paints(&self) -> Vec<P> {
        self.colln.get_paints()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.colln.has_paint_named(name)
    }
}

pub type CollnPaintCollnView<A, C, P, CID> = Rc<CollnPaintCollnViewCore<A, C, P, CID>>;

pub trait CollnPaintCollnViewInterface<A, C, P, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    fn pwo(&self) -> gtk::ScrolledWindow;
    fn create(colln: &CollnPaintColln<C, P, CID>) -> CollnPaintCollnView<A, C, P, CID>;
}

impl<A, C, P, CID> CollnPaintCollnViewInterface<A, C, P, CID> for CollnPaintCollnView<A, C, P, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }

    fn create(colln: &CollnPaintColln<C, P, CID>) -> CollnPaintCollnView<A, C, P, CID> {
        let len = P::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        for paint in colln.get_paints().iter() {
            list_store.append_row(&paint.tv_rows());
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let menu = gtk::Menu::new();
        let paint_info_item = gtk::MenuItem::new_with_label("Information");
        menu.append(&paint_info_item.clone());
        let add_paint_item = gtk::MenuItem::new_with_label("Add to Mixer");
        add_paint_item.set_visible(false);
        menu.append(&add_paint_item.clone());
        menu.show_all();

        let mspl = Rc::new(
            CollnPaintCollnViewCore::<A, C, P, CID> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                colln: colln.clone(),
                view: view,
                chosen_paint: RefCell::new(None),
                spec: PhantomData,
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
