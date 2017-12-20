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

use glib::signal::SignalHandlerId;
use gtk;
use gtk::prelude::*;

use pw_gix::dialogue::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::tree_view_column::*;

use struct_traits::*;
use basic_paint::*;

use super::display::*;
use super::hue_wheel::*;

// FACTORY
pub struct BasicPaintFactoryCore<C>
    where   C: CharacteristicsInterface,
{
    paints: RefCell<Vec<BasicPaint<C>>>,
}

impl<C> BasicPaintFactoryCore<C>
    where   C: CharacteristicsInterface,
{
    fn clear(&self) {
        self.paints.borrow_mut().clear()
    }

    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints.borrow().binary_search_by_key(
            &name.to_string(),
            |paint| paint.name()
        )
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<BasicPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None
        }
    }

    pub fn get_paints(&self) -> Vec<BasicPaint<C>> {
        let mut v: Vec<BasicPaint<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.clone())
        };
        v
    }

    pub fn get_paint_specs(&self) -> Vec<BasicPaintSpec<C>> {
        let mut v: Vec<BasicPaintSpec<C>> = Vec::new();
        for paint in self.paints.borrow().iter() {
            v.push(paint.get_spec())
        };
        v
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        match self.find_name(&spec.name) {
            Ok(_) => Err(PaintError::AlreadyExists(spec.name.clone())),
            Err(index) => {
                let paint = BasicPaint::<C>::from_spec(spec);
                self.paints.borrow_mut().insert(index, paint.clone());
                Ok(paint)
            }
        }
    }

    pub fn remove_paint(&self, paint: &BasicPaint<C>) {
        if let Ok(index) = self.find_name(&paint.name()) {
            let old_paint = self.paints.borrow_mut().remove(index);
            if old_paint != *paint {
                panic!("File: {} Line: {}", file!(), line!())
            }
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn replace_paint(&self, paint: &BasicPaint<C>, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        if paint.name() != spec.name && self.has_paint_named(&spec.name) {
            return Err(PaintError::AlreadyExists(spec.name.clone()))
        };
        self.remove_paint(paint);
        self.add_paint(spec)
    }
}

pub type BasicPaintFactory<C> = Rc<BasicPaintFactoryCore<C>>;

impl<C> SimpleCreation for BasicPaintFactory<C>
    where   C: CharacteristicsInterface,
{
    fn create() -> BasicPaintFactory<C> {
        let paints: RefCell<Vec<BasicPaint<C>>> = RefCell::new(Vec::new());
        Rc::new(BasicPaintFactoryCore::<C>{paints})
    }
}

// FACTORY VIEW
pub struct BasicPaintFactoryViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    paint_factory: BasicPaintFactory<C>,
    chosen_paint: RefCell<Option<BasicPaint<C>>>,
    spec: PhantomData<A>
}

impl<A, C> BasicPaintFactoryViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn clear(&self) {
        *self.chosen_paint.borrow_mut() = None;
        self.list_store.clear();
        self.paint_factory.clear();
    }

    pub fn get_paint_at(&self, posn: (f64, f64)) -> Option<BasicPaint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self.list_store.get_value(&iter, 0).get().unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    let paint = self.paint_factory.get_paint(&name).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    return Some(paint)
                }
            }
        };
        None
    }

    pub fn set_chosen_paint_from(&self, posn: (f64, f64)) -> Option<BasicPaint<C>> {
        if let Some(paint) = self.get_paint_at(posn) {
            *self.chosen_paint.borrow_mut() = Some(paint.clone());
            Some(paint)
        } else {
            *self.chosen_paint.borrow_mut() = None;
            None
        }
    }

    pub fn len(&self) -> usize {
        self.paint_factory.len()
    }

    pub fn get_paint(&self, name: &str) -> Option<BasicPaint<C>> {
        self.paint_factory.get_paint(name)
    }

    pub fn get_paints(&self) -> Vec<BasicPaint<C>> {
        self.paint_factory.get_paints()
    }

    pub fn get_paint_specs(&self) -> Vec<BasicPaintSpec<C>> {
        self.paint_factory.get_paint_specs()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.paint_factory.has_paint_named(name)
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        match self.paint_factory.add_paint(spec) {
            Ok(paint) => {
                self.list_store.append_row(&paint.tv_rows());
                Ok(paint)
            },
            Err(err) => Err(err)
        }
    }

    fn find_paint_named(&self, name: &str) -> Option<(i32, gtk::TreeIter)> {
        self.list_store.find_row_where(
            |list_store, iter| list_store.get_value(iter, 0).get() == Some(name)
        )
    }

    pub fn remove_paint(&self, paint: &BasicPaint<C>) {
        self.paint_factory.remove_paint(paint);
        if let Some((_, iter)) = self.find_paint_named(&paint.name()) {
            self.list_store.remove(&iter);
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn replace_paint(&self, paint: &BasicPaint<C>, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        let new_paint = self.paint_factory.replace_paint(paint, spec)?;
        if let Some((index, iter)) = self.find_paint_named(&paint.name()) {
            self.list_store.remove(&iter);
            self.list_store.insert_row(index, &new_paint.tv_rows());
            return Ok(new_paint);
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn connect_button_press_event<F: Fn(&gtk::TreeView, &gdk::EventButton) -> Inhibit + 'static>(&self, f: F) -> SignalHandlerId {
        self.view.connect_button_press_event(f)
    }
}

pub type BasicPaintFactoryView<A, C> = Rc<BasicPaintFactoryViewCore<A, C>>;

pub trait BasicPaintFactoryViewInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow;
    fn create() -> BasicPaintFactoryView<A, C>;
}

impl<A, C> BasicPaintFactoryViewInterface<A, C> for BasicPaintFactoryView<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }

    fn create() -> BasicPaintFactoryView<A, C> {
        let len = BasicPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
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
            BasicPaintFactoryViewCore::<A, C> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                paint_factory: BasicPaintFactory::<C>::create(),
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

// FACTORY POPUP
struct FactoryPopup {
    menu: gtk::Menu,
    show_paint_info_mi: gtk::MenuItem,
    edit_paint_mi: gtk::MenuItem,
    remove_paint_mi: gtk::MenuItem,
}

impl FactoryPopup {
    fn new() -> FactoryPopup {
        let fp = FactoryPopup {
            menu: gtk::Menu::new(),
            show_paint_info_mi: gtk::MenuItem::new_with_label("Paint Information"),
            edit_paint_mi: gtk::MenuItem::new_with_label("Edit Paint"),
            remove_paint_mi: gtk::MenuItem::new_with_label("Remove Paint"),
        };
        fp.menu.append(&fp.edit_paint_mi.clone());
        fp.menu.append(&fp.show_paint_info_mi.clone());
        fp.menu.append(&fp.remove_paint_mi.clone());
        fp.menu.show_all();

        fp.show_paint_info_mi.set_tooltip_text("Display this paint's information");
        fp.edit_paint_mi.set_tooltip_text("Select this paint for editing");
        fp.remove_paint_mi.set_tooltip_text("Remove this paint from the collection");

        fp
    }

    fn connect_show_paint_info<F: Fn(&gtk::MenuItem) + 'static>(&self, f: F) {
        self.show_paint_info_mi.connect_activate(f);
    }

    fn connect_edit_paint<F: Fn(&gtk::MenuItem) + 'static>(&self, f: F) {
        self.edit_paint_mi.connect_activate(f);
    }

    fn connect_remove_paint<F: Fn(&gtk::MenuItem) + 'static>(&self, f: F) {
        self.remove_paint_mi.connect_activate(f);
    }

    fn set_sensitivities(&self, edit_sensitivity: bool, other_sensitivity: bool) {
        self.edit_paint_mi.set_sensitive(edit_sensitivity);
        self.show_paint_info_mi.set_sensitive(other_sensitivity);
        self.remove_paint_mi.set_sensitive(other_sensitivity);
    }

    fn popup_at_event(&self, event: &gdk::EventButton) {
        self.menu.popup_easy(event.get_button(), event.get_time());
    }
}

// FACTORY DISPLAY CORE
pub struct BasicPaintFactoryDisplayCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    notebook: gtk::Notebook,
    paint_factory_view: BasicPaintFactoryView<A, C>,
    hue_attr_wheels: Vec<BasicPaintHueAttrWheel<C>>,
    chosen_paint: RefCell<Option<BasicPaint<C>>>,
    popup_menu: FactoryPopup,
    initiate_edit_ok: Cell<bool>,
    paint_dialogs: RefCell<HashMap<u32, BasicPaintDisplayDialog<A, C>>>,
    paint_removed_callbacks: RefCell<Vec<Box<Fn(&BasicPaint<C>)>>>,
    edit_paint_callbacks: RefCell<Vec<Box<Fn(&BasicPaint<C>)>>>,
}

impl<A, C> BasicPaintFactoryDisplayCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    pub fn clear(&self) {
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.close();
        };
        *self.chosen_paint.borrow_mut() = None;
        self.paint_factory_view.clear();
        for wheel in self.hue_attr_wheels.iter() {
            wheel.clear()
        };
    }

    pub fn set_initiate_edit_ok(&self, value: bool) {
        self.initiate_edit_ok.set(value);
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.set_response_sensitive(0, value);
        };
    }

    fn close_dialogs_for_paint(&self, paint: &BasicPaint<C>) {
        for dialog in self.paint_dialogs.borrow().values() {
            if dialog.paint() == *paint {
                dialog.close();
            }
        };
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        let paint = self.paint_factory_view.add_paint(spec)?;
        for wheel in self.hue_attr_wheels.iter() {
            wheel.add_paint(&paint)
        };
        Ok(paint)
    }

    pub fn replace_paint(&self, old_paint: &BasicPaint<C>, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError> {
        let new_paint = self.paint_factory_view.replace_paint(old_paint, spec)?;
        for wheel in self.hue_attr_wheels.iter() {
            wheel.replace_paint(old_paint, &new_paint)
        };
        self.close_dialogs_for_paint(old_paint);
        Ok(new_paint)
    }

    fn remove_paint(&self, paint: &BasicPaint<C>) {
        self.paint_factory_view.remove_paint(paint);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.remove_paint(paint)
        };
        self.close_dialogs_for_paint(paint);
        self.inform_paint_removed(paint);
    }

    fn remove_paint_after_confirmation(&self, paint: &BasicPaint<C>) {
        let question = format!("Confirm remove {}?", paint.name());
        if ask_confirm_action(parent_none(), &question, None) {
            self.remove_paint(paint)
        }
    }

    pub fn get_paint_specs(&self) -> Vec<BasicPaintSpec<C>> {
        self.paint_factory_view.get_paint_specs()
    }

    pub fn connect_edit_paint<F: 'static + Fn(&BasicPaint<C>)>(&self, callback: F) {
        self.edit_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_edit_paint(&self, paint: &BasicPaint<C>) {
        for callback in self.edit_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn connect_paint_removed<F: 'static + Fn(&BasicPaint<C>)>(&self, callback: F) {
        self.paint_removed_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_paint_removed(&self, paint: &BasicPaint<C>) {
        for callback in self.paint_removed_callbacks.borrow().iter() {
            callback(&paint);
        }
    }
}

// FACTORY DISPLAY
pub type BasicPaintFactoryDisplay<A, C> = Rc<BasicPaintFactoryDisplayCore<A, C>>;

impl<A, C> PackableWidgetObject<gtk::Notebook> for BasicPaintFactoryDisplayCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::Notebook {
        self.notebook.clone()
    }
}

impl<A, C> SimpleCreation for BasicPaintFactoryDisplay<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create() -> BasicPaintFactoryDisplay<A, C> {
        let notebook = gtk::Notebook::new();
        let paint_factory_view = BasicPaintFactoryView::<A, C>::create();
        notebook.append_page(&paint_factory_view.pwo(), Some(&gtk::Label::new("Paint List")));
        let mut hue_attr_wheels = Vec::new();
        for attr in A::scalar_attributes().iter() {
            let wheel = BasicPaintHueAttrWheel::<C>::create(*attr);
            let label_text = format!("Hue/{} Wheel", attr.to_string());
            let label = gtk::Label::new(Some(label_text.as_str()));
            notebook.append_page(&wheel.pwo(), Some(&label));
            hue_attr_wheels.push(wheel);
        }

        let bpf = Rc::new(
            BasicPaintFactoryDisplayCore::<A, C> {
                notebook: notebook,
                paint_factory_view: paint_factory_view,
                hue_attr_wheels: hue_attr_wheels,
                chosen_paint: RefCell::new(None),
                popup_menu: FactoryPopup::new(),
                initiate_edit_ok: Cell::new(false),
                paint_dialogs: RefCell::new(HashMap::new()),
                paint_removed_callbacks: RefCell::new(Vec::new()),
                edit_paint_callbacks: RefCell::new(Vec::new()),
            }
        );

        let bpf_c = bpf.clone();
        bpf.paint_factory_view.connect_button_press_event(
            move |_, event| {
                if event.get_button() == 3 {
                    if let Some(paint) = bpf_c.paint_factory_view.get_paint_at(event.get_position()) {
                        bpf_c.popup_menu.set_sensitivities(bpf_c.initiate_edit_ok.get(), true);
                        *bpf_c.chosen_paint.borrow_mut() = Some(paint);
                    } else {
                        bpf_c.popup_menu.set_sensitivities(false, false);
                        *bpf_c.chosen_paint.borrow_mut() = None;
                    };
                    bpf_c.popup_menu.popup_at_event(event);
                    return Inhibit(true)
                };
                Inhibit(false)
            }
        );

        for wheel in bpf.hue_attr_wheels.iter() {
            let bpf_c = bpf.clone();
            let wheel_c = wheel.clone();
            wheel.connect_button_press_event(
                move |_, event| {
                    if event.get_button() == 3 {
                        if let Some(paint) = wheel_c.get_paint_at(event.get_position()) {
                            bpf_c.popup_menu.set_sensitivities(bpf_c.initiate_edit_ok.get(), true);
                            *bpf_c.chosen_paint.borrow_mut() = Some(paint);
                        } else {
                            bpf_c.popup_menu.set_sensitivities(false, false);
                            *bpf_c.chosen_paint.borrow_mut() = None;
                        };
                        bpf_c.popup_menu.popup_at_event(event);
                        return Inhibit(true)
                    };
                    Inhibit(false)
                }
            );
        }

        let bpf_c = bpf.clone();
        bpf.popup_menu.connect_show_paint_info(
            move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    let bpf_c_c = bpf_c.clone();
                    let paint_c = paint.clone();
                    let edit_btn_spec = PaintDisplayButtonSpec {
                        label: "Edit".to_string(),
                        tooltip_text: "load this paint into the editor.".to_string(),
                        callback:  Box::new(move || bpf_c_c.inform_edit_paint(&paint_c))
                    };
                    let bpf_c_c = bpf_c.clone();
                    let paint_c = paint.clone();
                    let remove_btn_spec = PaintDisplayButtonSpec {
                        label: "Remove".to_string(),
                        tooltip_text: "Remove this paint from the collection.".to_string(),
                        callback:  Box::new(move || bpf_c_c.remove_paint_after_confirmation(&paint_c))
                    };
                    let dialog = BasicPaintDisplayDialog::<A, C>::create(&paint, None, vec![edit_btn_spec, remove_btn_spec]);
                    let bpf_c_c = bpf_c.clone();
                    dialog.connect_destroy(
                        move |id| { bpf_c_c.paint_dialogs.borrow_mut().remove(&id); }
                    );
                    bpf_c.paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            }
        );

        let bpf_c = bpf.clone();
        bpf.popup_menu.connect_edit_paint(
            move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    bpf_c.inform_edit_paint(paint)
                }
            }
        );

        let bpf_c = bpf.clone();
        bpf.popup_menu.connect_remove_paint(
            move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    bpf_c.remove_paint_after_confirmation(paint)
                }
            }
        );

        bpf
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
