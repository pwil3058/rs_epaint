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

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use num::Integer;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::rgb_math::rgb::*;

use display::*;
use paint::*;
use mixed_paint::*;

pub struct MixedPaintCollectionCore<C: CharacteristicsInterface> {
    last_mixture_id: Cell<u32>,
    paints: RefCell<Vec<MixedPaint<C>>>
}

impl<C: CharacteristicsInterface> MixedPaintCollectionCore<C> {
    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints.borrow().binary_search_by_key(
            &name.to_string(),
            |paint| paint.name()
        )
    }

    pub fn next_mixture_id(&self) -> u32 {
        self.last_mixture_id.get() + 1
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<Paint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(Paint::Mixed(self.paints.borrow()[index].clone())),
            Err(_) => None
        }
    }

    pub fn get_paints(&self) -> Vec<Paint<C>> {
        let mut v: Vec<Paint<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(Paint::Mixed(paint.clone()))
        };
        v
    }

    pub fn get_mixed_paint(&self, name: &str) -> Option<MixedPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn get_mixed_paints(&self) -> Vec<MixedPaint<C>> {
        let mut v: Vec<MixedPaint<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.clone())
        };
        v
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }

    pub fn add_paint(
        &self,
        notes: &str,
        components: &Vec<PaintComponent<C>>,
        matched_colour: Option<Colour>
    ) -> Result<MixedPaint<C>, PaintError> {
        let mut total_parts: u32 = 0;
        let mut gcd: u32 = 0;
        for component in components.iter() {
            gcd = gcd.gcd(&component.parts);
            total_parts += component.parts;
        }
        if gcd == 0 {
            return Err(PaintError::NoSubstantiveComponents)
        }
        total_parts /= gcd;
        let mut new_rgb: RGB = BLACK;
        let mut subst_components: Vec<PaintComponent<C>> = Vec::new();
        let mut new_c_floats: Vec<f64> = Vec::new();
        for _ in [0..C::tv_row_len()].iter() {
            new_c_floats.push(0.0);
        }
        for component in components.iter() {
            if component.parts > 0 {
                let subst_parts = component.parts / gcd;
                let subst_component = PaintComponent::<C>{parts:subst_parts, paint:component.paint.clone()};
                subst_components.push(subst_component);
                let weight: f64 = total_parts as f64 / subst_parts as f64;
                new_rgb += component.paint.rgb() * weight;
                let floats = component.paint.characteristics().to_floats();
                for (i, val) in new_c_floats.iter_mut().enumerate() {
                    *val = *val + floats[i] * weight;
                }
            }
        }
        let name_num = self.last_mixture_id.get() + 1;
        self.last_mixture_id.set(name_num);
        Ok(Rc::new(
            MixedPaintCore::<C> {
                colour: Colour::from(new_rgb),
                name: format!("Mix #{:03}", name_num),
                notes: notes.to_string(),
                characteristics: C::from_floats(&new_c_floats),
                matched_colour: matched_colour.clone(),
                components: Rc::new(subst_components)
            }
        ))
    }
}

pub type MixedPaintCollection<C> = Rc<MixedPaintCollectionCore<C>>;

pub trait MixedPaintCollectionInterface<C: CharacteristicsInterface> {
    fn create() -> MixedPaintCollection<C>;
}

impl<C> MixedPaintCollectionInterface<C> for MixedPaintCollection<C>
    where   C: CharacteristicsInterface
{
    fn create() -> MixedPaintCollection<C> {
        let last_mixture_id: Cell<u32> = Cell::new(0);
        let paints: RefCell<Vec<MixedPaint<C>>> = RefCell::new(Vec::new());
        Rc::new(MixedPaintCollectionCore::<C>{last_mixture_id, paints})
    }
}

pub struct MixedPaintCollectionViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    menu: gtk::Menu,
    paint_info_item: gtk::MenuItem,
    add_paint_item: gtk::MenuItem,
    collection: MixedPaintCollection<C>,
    chosen_paint: RefCell<Option<MixedPaint<C>>>,
    current_target: RefCell<Option<Colour>>,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&MixedPaint<C>)>>>,
    mixed_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
    spec: PhantomData<A>
}

impl<A, C> MixedPaintCollectionViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn get_mixed_paint_at(&self, posn: (f64, f64)) -> Option<MixedPaint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self.list_store.get_value(&iter, 0).get().unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    let paint = self.collection.get_mixed_paint(&name).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    return Some(paint)
                }
            }
        }
        None
    }

    fn inform_add_paint(&self, paint: &MixedPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        match ocolour {
            Some(colour) => {
                for dialog in self.mixed_paint_dialogs.borrow().values() {
                    dialog.set_current_target(Some(colour.clone()));
                };
                *self.current_target.borrow_mut() = Some(colour.clone())
            },
            None => {
                for dialog in self.mixed_paint_dialogs.borrow().values() {
                    dialog.set_current_target(None);
                };
                *self.current_target.borrow_mut() = None
            },
        }
    }

    pub fn add_paint(
        &self,
        notes: &str,
        components: &Vec<PaintComponent<C>>,
        matched_colour: Option<Colour>
    ) -> Result<MixedPaint<C>, PaintError> {
        match self.collection.add_paint(notes, components, matched_colour) {
            Ok(mixed_paint) => {
                self.list_store.append_row(&mixed_paint.tv_rows());
                Ok(mixed_paint)
            },
            Err(err) => Err(err)
        }
    }
}

pub type MixedPaintCollectionView<A, C> = Rc<MixedPaintCollectionViewCore<A, C>>;

pub trait MixedPaintCollectionViewInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow;
    fn create(collection: &MixedPaintCollection<C>) -> MixedPaintCollectionView<A, C>;
    fn connect_add_paint<F: 'static + Fn(&MixedPaint<C>)>(&self, callback: F);
}

impl<A, C> MixedPaintCollectionViewInterface<A, C> for MixedPaintCollectionView<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }

    fn create(collection: &MixedPaintCollection<C>) -> MixedPaintCollectionView<A, C> {
        let len = MixedPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&MIXED_PAINT_ROW_SPEC[0..len]);
        for paint in collection.get_mixed_paints().iter() {
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
            MixedPaintCollectionViewCore::<A, C> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                menu: menu,
                paint_info_item: paint_info_item.clone(),
                add_paint_item: add_paint_item.clone(),
                collection: collection.clone(),
                view: view,
                chosen_paint: RefCell::new(None),
                current_target: RefCell::new(None),
                add_paint_callbacks: RefCell::new(Vec::new()),
                mixed_paint_dialogs: RefCell::new(HashMap::new()),
                spec: PhantomData,
            }
        );

        mspl.view.append_column(&simple_text_column("Name", 0, 0, 6, 7, -1, true));
        mspl.view.append_column(&simple_text_column("Match?", -1, 14, 15, -1, 50, true));
        mspl.view.append_column(&simple_text_column("Notes", 1, 1, 6, 7, -1, true));
        for col in A::tv_columns() {
            mspl.view.append_column(&col);
        }
        for col in C::tv_columns(14) {
            mspl.view.append_column(&col);
        }

        mspl.view.show_all();

        mspl.scrolled_window.add(&mspl.view.clone());
        mspl.scrolled_window.show_all();

        let mspl_c = mspl.clone();
        mspl.view.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let o_paint = mspl_c.get_mixed_paint_at(event.get_position());
                        mspl_c.paint_info_item.set_sensitive(o_paint.is_some());
                        mspl_c.add_paint_item.set_sensitive(o_paint.is_some());
                        let have_listeners = mspl_c.add_paint_callbacks.borrow().len() > 0;
                        mspl_c.add_paint_item.set_visible(have_listeners);
                        *mspl_c.chosen_paint.borrow_mut() = o_paint;
                        mspl_c.menu.popup_at_pointer(None);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );

        let mspl_c = mspl.clone();
        add_paint_item.connect_activate(
            move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    mspl_c.inform_add_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );

        let mspl_c = mspl.clone();
        paint_info_item.clone().connect_activate(
            move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    let target = if let Some(ref colour) = *mspl_c.current_target.borrow() {
                        Some(colour.clone())
                    } else {
                        None
                    };
                    let have_listeners = mspl_c.add_paint_callbacks.borrow().len() > 0;
                    let buttons = if have_listeners {
                        let mspl_c_c = mspl_c.clone();
                        let paint_c = paint.clone();
                        let spec = PaintDisplayButtonSpec {
                            label: "Add".to_string(),
                            tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                            callback:  Box::new(move || mspl_c_c.inform_add_paint(&paint_c))
                        };
                        vec![spec]
                    } else {
                        vec![]
                    };
                    let dialog = PaintDisplayDialog::<A, C>::mixed_create(
                        &paint,
                        target,
                        None,
                        buttons
                    );
                    let mspl_c_c = mspl_c.clone();
                    dialog.connect_destroy(
                        move |id| { mspl_c_c.mixed_paint_dialogs.borrow_mut().remove(&id); }
                    );
                    mspl_c.mixed_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            }
        );

        mspl
    }

    fn connect_add_paint<F: 'static + Fn(&MixedPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks.borrow_mut().push(Box::new(callback))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}