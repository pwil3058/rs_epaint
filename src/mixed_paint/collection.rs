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
use pw_gix::gtkx::menu::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::rgb_math::rgb::*;
use pw_gix::wrapper::*;

use crate::basic_paint::*;
use crate::dialogue::*;
use crate::error::*;

use super::components::*;
use super::display::*;
use super::target::TargetColourInterface;
use super::*;

pub struct MixedPaintFactoryCore<C: CharacteristicsInterface> {
    last_mixture_id: Cell<u32>,
    paints: RefCell<Vec<MixedPaint<C>>>,
}

impl<C: CharacteristicsInterface> MixedPaintFactoryCore<C> {
    fn find_name(&self, name: &str) -> Result<usize, usize> {
        let result = self
            .paints
            .borrow()
            .binary_search_by_key(&name.to_string(), |paint| paint.name());
        result
    }

    pub fn next_mixture_id(&self) -> u32 {
        self.last_mixture_id.get() + 1
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<MixedPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None,
        }
    }

    pub fn get_paints(&self) -> Vec<MixedPaint<C>> {
        self.paints.borrow().iter().map(|p| p.clone()).collect()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }

    pub fn add_paint(
        &self,
        notes: &str,
        sp_components: Vec<(SeriesPaint<C>, u32)>,
        mp_components: Vec<(MixedPaint<C>, u32)>,
        matched_colour: Option<Colour>,
    ) -> Result<MixedPaint<C>, PaintError<C>> {
        let mut parts: Vec<u32> = sp_components.iter().map(|c| c.1).collect();
        parts.extend(mp_components.iter().map(|c| c.1));
        let gcd: u32 = parts.iter().fold(0, |gcd, p| gcd.gcd(&p));
        if gcd == 0 {
            return Err(PaintErrorType::NoSubstantiveComponents.into());
        }
        let mut total_parts: u32 = parts.iter().sum();
        total_parts /= gcd;
        let mut new_rgb: RGB = BLACK;
        let mut p_components: Vec<PaintComponent<C>> = Vec::new();
        let mut new_c_floats = vec![0.0_f64; C::tv_row_len()];
        for (series_paint, mut parts) in sp_components {
            if parts > 0 {
                parts /= gcd;
                let weight: f64 = parts as f64 / total_parts as f64;
                new_rgb += series_paint.rgb() * weight;
                let floats = series_paint.characteristics().to_floats();
                for (i, val) in new_c_floats.iter_mut().enumerate() {
                    *val = *val + floats[i] * weight;
                }
                let paint = Paint::Series(series_paint);
                p_components.push(PaintComponent { parts, paint });
            }
        }
        for (mixed_paint, mut parts) in mp_components {
            if parts > 0 {
                parts /= gcd;
                let weight: f64 = parts as f64 / total_parts as f64;
                new_rgb += mixed_paint.rgb() * weight;
                let floats = mixed_paint.characteristics().to_floats();
                for (i, val) in new_c_floats.iter_mut().enumerate() {
                    *val = *val + floats[i] * weight;
                }
                let paint = Paint::Mixed(mixed_paint);
                p_components.push(PaintComponent { parts, paint });
            }
        }
        let name_num = self.last_mixture_id.get() + 1;
        let target_colour = if let Some(ref colour) = matched_colour {
            let name = format!("Target #{:03}", name_num);
            Some(TargetColour::create(colour, &name, notes))
        } else {
            None
        };
        self.last_mixture_id.set(name_num);
        let mixed_paint = Rc::new(MixedPaintCore::<C> {
            colour: Colour::from(new_rgb),
            name: format!("Mix #{:03}", name_num),
            notes: RefCell::new(notes.to_string()),
            characteristics: C::from_floats(&new_c_floats),
            target_colour: target_colour,
            components: Rc::new(p_components),
        });
        self.paints.borrow_mut().push(mixed_paint.clone());
        Ok(mixed_paint)
    }

    pub fn remove_paint(&self, paint: &MixedPaint<C>) -> Result<(), PaintError<C>> {
        let users = self.mixed_paints_using(&Paint::Mixed(paint.clone()));
        if users.len() > 0 {
            return Err(PaintErrorType::BeingUsedBy(users).into());
        }
        if let Ok(index) = self.find_name(&paint.name()) {
            let old_paint = self.paints.borrow_mut().remove(index);
            if old_paint != *paint {
                panic!("File: {} Line: {}", file!(), line!())
            }
        } else {
            return Err(PaintErrorType::NotFound(paint.name()).into());
        }
        Ok(())
    }

    pub fn series_paints_used(&self) -> Vec<SeriesPaint<C>> {
        let mut spu: Vec<SeriesPaint<C>> = Vec::new();
        for mixed_paint in self.paints.borrow().iter() {
            for series_paint in mixed_paint.series_paints_used().iter() {
                if let Err(index) = spu.binary_search(series_paint) {
                    // NB: Ok case means paint already in the list
                    spu.insert(index, series_paint.clone())
                }
            }
        }

        spu
    }

    pub fn mixed_paints_using(&self, paint: &Paint<C>) -> Vec<MixedPaint<C>> {
        self.paints
            .borrow()
            .iter()
            .filter(|p| p.uses_paint(paint))
            .map(|m| m.clone())
            .collect()
    }

    pub fn mixed_paints_using_series_paint(&self, paint: &SeriesPaint<C>) -> Vec<MixedPaint<C>> {
        self.paints
            .borrow()
            .iter()
            .filter(|p| p.uses_series_paint(paint))
            .map(|m| m.clone())
            .collect()
    }
}

pub type MixedPaintFactory<C> = Rc<MixedPaintFactoryCore<C>>;

pub trait MixedPaintFactoryInterface<C: CharacteristicsInterface> {
    fn create() -> MixedPaintFactory<C>;
}

impl<C> MixedPaintFactoryInterface<C> for MixedPaintFactory<C>
where
    C: CharacteristicsInterface,
{
    fn create() -> MixedPaintFactory<C> {
        let last_mixture_id: Cell<u32> = Cell::new(0);
        let paints: RefCell<Vec<MixedPaint<C>>> = RefCell::new(Vec::new());
        Rc::new(MixedPaintFactoryCore::<C> {
            last_mixture_id,
            paints,
        })
    }
}

pub type MixedPaintComponentBox<A, C> =
    PaintComponentsBox<A, C, MixedPaint<C>, MixedPaintDisplayDialog<A, C>>;

pub struct MixedPaintCollectionWidgetCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    vbox: gtk::Box,
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    popup_menu: WrappedMenu,
    factory: MixedPaintFactory<C>,
    components: MixedPaintComponentBox<A, C>,
    chosen_paint: RefCell<Option<MixedPaint<C>>>,
    current_target: RefCell<Option<Colour>>,
    add_paint_callbacks: RefCell<Vec<Box<dyn Fn(&MixedPaint<C>)>>>,
    remove_paint_callbacks: RefCell<Vec<Box<dyn Fn(&MixedPaint<C>)>>>,
    mixed_paint_dialogs: RefCell<HashMap<u32, MixedPaintDisplayDialog<A, C>>>,
    spec: PhantomData<A>,
}

impl<A, C> MixedPaintCollectionWidgetCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    pub fn next_mixture_id(&self) -> u32 {
        self.factory.next_mixture_id()
    }

    fn get_mixed_paint_at(&self, posn: (f64, f64)) -> Option<MixedPaint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self
                        .list_store
                        .get_value(&iter, MP_NAME)
                        .get()
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    let paint = self
                        .factory
                        .get_paint(&name)
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    return Some(paint);
                }
            }
        }
        None
    }

    pub fn connect_add_paint<F: 'static + Fn(&MixedPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_add_paint(&self, paint: &MixedPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn connect_remove_paint<F: 'static + Fn(&MixedPaint<C>)>(&self, callback: F) {
        self.remove_paint_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_remove_paint(&self, paint: &MixedPaint<C>) {
        for callback in self.remove_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        for dialog in self.mixed_paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        }
        self.components.set_current_target(o_colour);
        if let Some(colour) = o_colour {
            *self.current_target.borrow_mut() = Some(colour.clone())
        } else {
            *self.current_target.borrow_mut() = None
        }
    }

    pub fn add_paint(
        &self,
        notes: &str,
        sp_components: Vec<(SeriesPaint<C>, u32)>,
        mp_components: Vec<(MixedPaint<C>, u32)>,
        matched_colour: Option<Colour>,
    ) -> Result<MixedPaint<C>, PaintError<C>> {
        match self
            .factory
            .add_paint(notes, sp_components, mp_components, matched_colour)
        {
            Ok(mixed_paint) => {
                self.list_store.append_row(&mixed_paint.tv_rows());
                Ok(mixed_paint)
            }
            Err(err) => Err(err),
        }
    }

    fn find_paint_named(&self, name: &str) -> Option<(i32, gtk::TreeIter)> {
        self.list_store
            .find_row_where(|list_store, iter| list_store.get_value(iter, 0).get() == Some(name))
    }

    fn set_notes_for_paint_at(&self, iter: &gtk::TreeIter, new_notes: &str) {
        let o_paint_name: Option<String> = self.list_store.get_value(iter, MP_NAME).get();
        if let Some(ref paint_name) = o_paint_name {
            if let Some(paint) = self.factory.get_paint(paint_name) {
                paint.set_notes(new_notes);
                self.list_store
                    .set_value(iter, MP_NOTES as u32, &new_notes.into());
            } else {
                panic!("File: {} Line: {}", file!(), line!())
            }
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn remove_paint(&self, paint: &MixedPaint<C>) -> Result<(), PaintError<C>> {
        if self.components.is_being_used(paint) {
            return Err(PaintErrorType::PartOfCurrentMixture.into());
        };
        self.factory.remove_paint(paint)?;
        self.components.remove_paint(paint);
        if let Some((_, iter)) = self.find_paint_named(&paint.name()) {
            self.list_store.remove(&iter);
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        };
        Ok(())
    }

    pub fn series_paints_used(&self) -> Vec<SeriesPaint<C>> {
        self.factory.series_paints_used()
    }

    pub fn get_paints(&self) -> Vec<MixedPaint<C>> {
        self.factory.get_paints()
    }

    pub fn mixed_paints_using_series_paint(&self, paint: &SeriesPaint<C>) -> Vec<MixedPaint<C>> {
        self.factory.mixed_paints_using_series_paint(paint)
    }

    // Components interface
    pub fn components(&self) -> MixedPaintComponentBox<A, C> {
        self.components.clone()
    }
}

impl_widget_wrapper!(vbox: gtk::Box, MixedPaintCollectionWidgetCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
);

pub type MixedPaintCollectionWidget<A, C> = Rc<MixedPaintCollectionWidgetCore<A, C>>;

pub trait MixedPaintCollectionWidgetInterface<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(mixing_mode: MixingMode) -> MixedPaintCollectionWidget<A, C>;
}

impl<A, C> MixedPaintCollectionWidgetInterface<A, C> for MixedPaintCollectionWidget<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(mixing_mode: MixingMode) -> MixedPaintCollectionWidget<A, C> {
        let len = MixedPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&MIXED_PAINT_ROW_SPEC[0..len]);
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::Single);

        let adj: Option<&gtk::Adjustment> = None;
        let mspl = Rc::new(MixedPaintCollectionWidgetCore::<A, C> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 2),
            scrolled_window: gtk::ScrolledWindow::new(adj, adj),
            list_store: list_store,
            popup_menu: WrappedMenu::new(&vec![]),
            factory: MixedPaintFactory::create(),
            components: MixedPaintComponentBox::<A, C>::create_with(4, true),
            view: view,
            chosen_paint: RefCell::new(None),
            current_target: RefCell::new(None),
            add_paint_callbacks: RefCell::new(Vec::new()),
            remove_paint_callbacks: RefCell::new(Vec::new()),
            mixed_paint_dialogs: RefCell::new(HashMap::new()),
            spec: PhantomData,
        });

        mspl.view.append_column(&simple_text_column(
            "Name", MP_NAME, MP_NAME, MP_RGB, MP_RGB_FG, -1, true,
        ));
        if mixing_mode == MixingMode::MatchTarget {
            mspl.view.append_column(&simple_text_column(
                "Match?",
                -1,
                MP_MATCHED_ANGLE,
                MP_MATCHED_RGB,
                -1,
                50,
                true,
            ));
        };
        let mspl_c = mspl.clone();
        let notes_col = editable_text_column(
            "Notes",
            MP_NOTES,
            MP_NOTES,
            MP_RGB,
            MP_RGB_FG,
            -1,
            true,
            move |_, tree_path, new_notes| {
                if let Some(ref iter) = mspl_c.list_store.get_iter(&tree_path) {
                    mspl_c.set_notes_for_paint_at(iter, new_notes);
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            },
        );
        mspl.view.append_column(&notes_col);
        for col in A::tv_columns() {
            mspl.view.append_column(&col);
        }
        for col in C::tv_columns(MP_CHARS_0) {
            mspl.view.append_column(&col);
        }

        mspl.view.show_all();

        mspl.scrolled_window.add(&mspl.view.clone());
        mspl.scrolled_window.show_all();
        mspl.vbox
            .pack_start(&mspl.components.pwo(), false, false, 0);
        mspl.vbox.pack_start(&mspl.scrolled_window, true, true, 0);
        mspl.vbox.show_all();

        let mspl_c = mspl.clone();
        mspl.popup_menu
            .append_item(
                "info",
                "Paint Information",
                "Display this paint's information",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    let target_colour = mspl_c.current_target.borrow().clone();
                    let target = if let Some(ref colour) = target_colour {
                        Some(colour)
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
                            callback: Box::new(move || mspl_c_c.inform_add_paint(&paint_c)),
                        };
                        vec![spec]
                    } else {
                        vec![]
                    };
                    let dialog =
                        MixedPaintDisplayDialog::<A, C>::create(&paint, target, &mspl_c, buttons);
                    let mspl_c_c = mspl_c.clone();
                    dialog.connect_destroyed(move |id| {
                        mspl_c_c.mixed_paint_dialogs.borrow_mut().remove(&id);
                    });
                    mspl_c
                        .mixed_paint_dialogs
                        .borrow_mut()
                        .insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            });

        let mspl_c = mspl.clone();
        mspl.popup_menu
            .append_item("add", "Add to Mixer", "Add this paint to the mixer palette")
            .connect_activate(move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    mspl_c.components.add_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            });

        let mspl_c = mspl.clone();
        mspl.popup_menu
            .append_item(
                "delete",
                "Delete",
                "Remove this paint from the collection of mixed paints",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *mspl_c.chosen_paint.borrow() {
                    mspl_c.inform_remove_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            });

        let mspl_c = mspl.clone();
        mspl.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress {
                if event.get_button() == 3 {
                    let o_paint = mspl_c.get_mixed_paint_at(event.get_position());
                    mspl_c
                        .popup_menu
                        .set_sensitivities(o_paint.is_some(), &["info"]);
                    mspl_c
                        .popup_menu
                        .set_sensitivities(o_paint.is_some(), &["add", "delete"]);
                    let have_listeners = mspl_c.components().has_listeners();
                    mspl_c.popup_menu.set_visibilities(have_listeners, &["add"]);
                    let have_listeners = mspl_c.remove_paint_callbacks.borrow().len() > 0;
                    mspl_c
                        .popup_menu
                        .set_visibilities(have_listeners, &["delete"]);
                    *mspl_c.chosen_paint.borrow_mut() = o_paint;
                    mspl_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                } else if event.get_button() == 2 {
                    mspl_c.view.get_selection().unselect_all();
                    return Inhibit(true);
                }
            }
            Inhibit(false)
        });

        mspl
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

}
