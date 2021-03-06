// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use pw_gix::{
    glib::signal::SignalHandlerId,
    gtk::{self, prelude::*},
    gtkx::{list_store::*, menu::*, tree_view_column::*},
    wrapper::*,
};

use crate::basic_paint::*;
pub use crate::struct_traits::SimpleCreation;

use super::display::*;
use super::hue_wheel::*;

// FACTORY
pub struct BasicPaintFactoryCore<C>
where
    C: CharacteristicsInterface,
{
    paints: RefCell<Vec<BasicPaint<C>>>,
}

impl<C> BasicPaintFactoryCore<C>
where
    C: CharacteristicsInterface,
{
    fn clear(&self) {
        self.paints.borrow_mut().clear()
    }

    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints
            .borrow()
            .binary_search_by_key(&name.to_string(), |paint| paint.name())
    }

    pub fn len(&self) -> usize {
        self.paints.borrow().len()
    }

    pub fn get_paint(&self, name: &str) -> Option<BasicPaint<C>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints.borrow()[index].clone()),
            Err(_) => None,
        }
    }

    pub fn get_paints(&self) -> Vec<BasicPaint<C>> {
        self.paints.borrow().iter().map(|p| p.clone()).collect()
    }

    pub fn get_paint_specs(&self) -> Vec<BasicPaintSpec<C>> {
        self.paints.borrow().iter().map(|p| p.get_spec()).collect()
    }

    pub fn matches_paint_specs(&self, specs: &Vec<BasicPaintSpec<C>>) -> bool {
        let paints = self.paints.borrow();
        if paints.len() != specs.len() {
            false
        } else {
            for spec in specs.iter() {
                if let Ok(index) = self.find_name(&spec.name) {
                    if !(paints[index].matches_spec(spec)) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError<C>> {
        match self.find_name(&spec.name) {
            Ok(_) => Err(PaintErrorType::AlreadyExists(spec.name.clone()).into()),
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

    pub fn replace_paint(
        &self,
        paint: &BasicPaint<C>,
        spec: &BasicPaintSpec<C>,
    ) -> Result<BasicPaint<C>, PaintError<C>> {
        if paint.name() != spec.name && self.has_paint_named(&spec.name) {
            return Err(PaintErrorType::AlreadyExists(spec.name.clone()).into());
        };
        self.remove_paint(paint);
        self.add_paint(spec)
    }
}

pub type BasicPaintFactory<C> = Rc<BasicPaintFactoryCore<C>>;

impl<C> SimpleCreation for BasicPaintFactory<C>
where
    C: CharacteristicsInterface,
{
    fn create() -> BasicPaintFactory<C> {
        let paints: RefCell<Vec<BasicPaint<C>>> = RefCell::new(Vec::new());
        Rc::new(BasicPaintFactoryCore::<C> { paints })
    }
}

// FACTORY VIEW
#[derive(PWO, Wrapper)]
pub struct BasicPaintFactoryViewCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    paint_factory: BasicPaintFactory<C>,
    chosen_paint: RefCell<Option<BasicPaint<C>>>,
    spec: PhantomData<A>,
}

impl<A, C> BasicPaintFactoryViewCore<A, C>
where
    A: ColourAttributesInterface + 'static,
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
                    let name: String = self
                        .list_store
                        .get_value(&iter, 0)
                        .get()
                        .unwrap()
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    let paint = self
                        .paint_factory
                        .get_paint(&name)
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    return Some(paint);
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

    pub fn matches_paint_specs(&self, specs: &Vec<BasicPaintSpec<C>>) -> bool {
        self.paint_factory.matches_paint_specs(specs)
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.paint_factory.has_paint_named(name)
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError<C>> {
        match self.paint_factory.add_paint(spec) {
            Ok(paint) => {
                self.list_store.append_row(&paint.tv_rows());
                Ok(paint)
            }
            Err(err) => Err(err),
        }
    }

    fn find_paint_named(&self, name: &str) -> Option<(i32, gtk::TreeIter)> {
        self.list_store.find_row_where(|list_store, iter| {
            list_store.get_value(iter, 0).get().unwrap() == Some(name)
        })
    }

    pub fn remove_paint(&self, paint: &BasicPaint<C>) {
        self.paint_factory.remove_paint(paint);
        if let Some((_, iter)) = self.find_paint_named(&paint.name()) {
            self.list_store.remove(&iter);
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn replace_paint(
        &self,
        paint: &BasicPaint<C>,
        spec: &BasicPaintSpec<C>,
    ) -> Result<BasicPaint<C>, PaintError<C>> {
        let new_paint = self.paint_factory.replace_paint(paint, spec)?;
        if let Some((index, iter)) = self.find_paint_named(&paint.name()) {
            self.list_store.remove(&iter);
            self.list_store.insert_row(index, &new_paint.tv_rows());
            return Ok(new_paint);
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    pub fn connect_button_press_event<
        F: Fn(&gtk::TreeView, &gdk::EventButton) -> Inhibit + 'static,
    >(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.view.connect_button_press_event(f)
    }
}

pub type BasicPaintFactoryView<A, C> = Rc<BasicPaintFactoryViewCore<A, C>>;

pub trait BasicPaintFactoryViewInterface<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create() -> BasicPaintFactoryView<A, C>;
}

impl<A, C> BasicPaintFactoryViewInterface<A, C> for BasicPaintFactoryView<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create() -> BasicPaintFactoryView<A, C> {
        let len = BasicPaint::<C>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        let view = gtk::TreeView::with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let adj: Option<&gtk::Adjustment> = None;
        let mspl = Rc::new(BasicPaintFactoryViewCore::<A, C> {
            scrolled_window: gtk::ScrolledWindow::new(adj, adj),
            list_store: list_store,
            paint_factory: BasicPaintFactory::<C>::create(),
            view: view,
            chosen_paint: RefCell::new(None),
            spec: PhantomData,
        });

        mspl.view.append_column(&simple_text_column(
            "Name", SP_NAME, SP_NAME, SP_RGB, SP_RGB_FG, -1, true,
        ));
        mspl.view.append_column(&simple_text_column(
            "Notes", SP_NOTES, SP_NOTES, SP_RGB, SP_RGB_FG, -1, true,
        ));
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

// FACTORY DISPLAY CORE
#[derive(PWO, Wrapper)]
pub struct BasicPaintFactoryDisplayCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    notebook: gtk::Notebook,
    paint_factory_view: BasicPaintFactoryView<A, C>,
    hue_attr_wheels: Vec<BasicPaintHueAttrWheel<C>>,
    chosen_paint: RefCell<Option<BasicPaint<C>>>,
    popup_menu: WrappedMenu,
    initiate_edit_ok: Cell<bool>,
    paint_dialogs: RefCell<HashMap<u32, BasicPaintDisplayDialog<A, C>>>,
    paint_removed_callbacks: RefCell<Vec<Box<dyn Fn(&BasicPaint<C>)>>>,
    edit_paint_callbacks: RefCell<Vec<Box<dyn Fn(&BasicPaint<C>)>>>,
}

impl<A, C> BasicPaintFactoryDisplayCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    pub fn len(&self) -> usize {
        self.paint_factory_view.len()
    }

    pub fn clear(&self) {
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.close();
        }
        *self.chosen_paint.borrow_mut() = None;
        self.paint_factory_view.clear();
        for wheel in self.hue_attr_wheels.iter() {
            wheel.clear()
        }
    }

    pub fn set_initiate_edit_ok(&self, value: bool) {
        self.initiate_edit_ok.set(value);
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.set_response_sensitive(gtk::ResponseType::Other(0), value);
        }
    }

    fn close_dialogs_for_paint(&self, paint: &BasicPaint<C>) {
        for dialog in self
            .paint_dialogs
            .borrow()
            .values()
            .filter(|d| d.paint() == *paint)
        {
            dialog.close();
        }
    }

    pub fn add_paint(&self, spec: &BasicPaintSpec<C>) -> Result<BasicPaint<C>, PaintError<C>> {
        let paint = self.paint_factory_view.add_paint(spec)?;
        for wheel in self.hue_attr_wheels.iter() {
            wheel.add_paint(&paint)
        }
        Ok(paint)
    }

    pub fn replace_paint(
        &self,
        old_paint: &BasicPaint<C>,
        spec: &BasicPaintSpec<C>,
    ) -> Result<BasicPaint<C>, PaintError<C>> {
        let new_paint = self.paint_factory_view.replace_paint(old_paint, spec)?;
        for wheel in self.hue_attr_wheels.iter() {
            wheel.replace_paint(old_paint, &new_paint)
        }
        self.close_dialogs_for_paint(old_paint);
        Ok(new_paint)
    }

    fn remove_paint(&self, paint: &BasicPaint<C>) {
        self.paint_factory_view.remove_paint(paint);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.remove_paint(paint)
        }
        self.close_dialogs_for_paint(paint);
        self.inform_paint_removed(paint);
    }

    fn remove_paint_after_confirmation(&self, paint: &BasicPaint<C>) {
        let question = format!("Confirm remove {}?", paint.name());
        if self.ask_confirm_action(&question, None) {
            self.remove_paint(paint)
        }
    }

    pub fn get_paint_specs(&self) -> Vec<BasicPaintSpec<C>> {
        self.paint_factory_view.get_paint_specs()
    }

    pub fn matches_paint_specs(&self, specs: &Vec<BasicPaintSpec<C>>) -> bool {
        self.paint_factory_view.matches_paint_specs(specs)
    }

    pub fn connect_edit_paint<F: 'static + Fn(&BasicPaint<C>)>(&self, callback: F) {
        self.edit_paint_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_edit_paint(&self, paint: &BasicPaint<C>) {
        for callback in self.edit_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn connect_paint_removed<F: 'static + Fn(&BasicPaint<C>)>(&self, callback: F) {
        self.paint_removed_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_paint_removed(&self, paint: &BasicPaint<C>) {
        for callback in self.paint_removed_callbacks.borrow().iter() {
            callback(&paint);
        }
    }
}

// FACTORY DISPLAY
pub type BasicPaintFactoryDisplay<A, C> = Rc<BasicPaintFactoryDisplayCore<A, C>>;

impl<A, C> SimpleCreation for BasicPaintFactoryDisplay<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create() -> BasicPaintFactoryDisplay<A, C> {
        let notebook = gtk::Notebook::new();
        notebook.set_scrollable(true);
        notebook.popup_enable();
        let paint_factory_view = BasicPaintFactoryView::<A, C>::create();
        notebook.append_page(
            &paint_factory_view.pwo(),
            Some(&gtk::Label::new(Some("Paint List"))),
        );
        let mut hue_attr_wheels = Vec::new();
        for attr in A::scalar_attributes().iter() {
            let wheel = BasicPaintHueAttrWheel::<C>::create(*attr);
            let label_text = format!("Hue/{} Wheel", attr.to_string());
            let label = gtk::Label::new(Some(label_text.as_str()));
            notebook.append_page(&wheel.pwo(), Some(&label));
            hue_attr_wheels.push(wheel);
        }

        let bpf = Rc::new(BasicPaintFactoryDisplayCore::<A, C> {
            notebook: notebook,
            paint_factory_view: paint_factory_view,
            hue_attr_wheels: hue_attr_wheels,
            chosen_paint: RefCell::new(None),
            popup_menu: WrappedMenu::new(&vec![]),
            initiate_edit_ok: Cell::new(false),
            paint_dialogs: RefCell::new(HashMap::new()),
            paint_removed_callbacks: RefCell::new(Vec::new()),
            edit_paint_callbacks: RefCell::new(Vec::new()),
        });

        let bpf_c = bpf.clone();
        bpf.popup_menu
            .append_item("edit", "Edit Paint", "Select this paint for editing")
            .connect_activate(move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    bpf_c.inform_edit_paint(paint)
                }
            });

        let bpf_c = bpf.clone();
        bpf.popup_menu
            .append_item(
                "info",
                "Paint Information",
                "Display this paint's information",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    let bpf_c_c = bpf_c.clone();
                    let paint_c = paint.clone();
                    let edit_btn_spec = PaintDisplayButtonSpec {
                        label: "Edit".to_string(),
                        tooltip_text: "load this paint into the editor.".to_string(),
                        callback: Box::new(move || bpf_c_c.inform_edit_paint(&paint_c)),
                    };
                    let bpf_c_c = bpf_c.clone();
                    let paint_c = paint.clone();
                    let remove_btn_spec = PaintDisplayButtonSpec {
                        label: "Remove".to_string(),
                        tooltip_text: "Remove this paint from the collection.".to_string(),
                        callback: Box::new(move || {
                            bpf_c_c.remove_paint_after_confirmation(&paint_c)
                        }),
                    };
                    let dialog = BasicPaintDisplayDialog::<A, C>::create(
                        &paint,
                        &bpf_c,
                        vec![edit_btn_spec, remove_btn_spec],
                    );
                    let bpf_c_c = bpf_c.clone();
                    dialog.connect_destroyed(move |id| {
                        bpf_c_c.paint_dialogs.borrow_mut().remove(&id);
                    });
                    bpf_c
                        .paint_dialogs
                        .borrow_mut()
                        .insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            });

        let bpf_c = bpf.clone();
        bpf.popup_menu
            .append_item(
                "remove",
                "Remove Paint",
                "Remove this paint from the collection",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *bpf_c.chosen_paint.borrow() {
                    bpf_c.remove_paint_after_confirmation(paint)
                }
            });

        let bpf_c = bpf.clone();
        bpf.paint_factory_view
            .connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    if let Some(paint) = bpf_c.paint_factory_view.get_paint_at(event.get_position())
                    {
                        bpf_c
                            .popup_menu
                            .set_sensitivities(bpf_c.initiate_edit_ok.get(), &["edit"]);
                        bpf_c
                            .popup_menu
                            .set_sensitivities(true, &["info", "remove"]);
                        *bpf_c.chosen_paint.borrow_mut() = Some(paint);
                    } else {
                        bpf_c
                            .popup_menu
                            .set_sensitivities(false, &["edit", "info", "remove"]);
                        *bpf_c.chosen_paint.borrow_mut() = None;
                    };
                    bpf_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                };
                Inhibit(false)
            });

        for wheel in bpf.hue_attr_wheels.iter() {
            let bpf_c = bpf.clone();
            let wheel_c = wheel.clone();
            wheel.connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    if let Some(paint) = wheel_c.get_paint_at(event.get_position()) {
                        bpf_c
                            .popup_menu
                            .set_sensitivities(bpf_c.initiate_edit_ok.get(), &["edit"]);
                        bpf_c
                            .popup_menu
                            .set_sensitivities(true, &["info", "remove"]);
                        *bpf_c.chosen_paint.borrow_mut() = Some(paint);
                    } else {
                        bpf_c
                            .popup_menu
                            .set_sensitivities(false, &["edit", "info", "remove"]);
                        *bpf_c.chosen_paint.borrow_mut() = None;
                    };
                    bpf_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                };
                Inhibit(false)
            });
        }

        bpf
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
