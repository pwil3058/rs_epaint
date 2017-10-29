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
//use std::convert::From;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;
use gtk::MenuExt;

use gtkx::coloured::*;
use paint::*;
use paint::colour_mix::*;
use pwo::*;

pub trait PaintPartsSpinButtonInterface<C>: PackableWidgetInterface
    where   C: Hash + Clone + PartialEq + Copy
{
    type PaintPartsSpinButtonType;

    fn create_with(paint: &Paint<C>, sensitive:bool) -> Self::PaintPartsSpinButtonType;
    fn get_parts(&self) -> u32;
    fn set_parts(&self, parts: u32);
    fn divide_parts(&self, divisor: u32);
    fn get_colour_component(&self) -> ColourComponent;
    fn get_paint_component(&self) -> PaintComponent<C>;
    fn set_sensitive(&self, sensitive: bool);
    fn connect_parts_changed<F: 'static + Fn(u32)>(&self, callback: F);
    fn inform_parts_changed(&self);
    fn connect_remove_me<F: 'static + Fn(&PaintPartsSpinButton<C>)>(&self, callback: F);
    fn inform_remove_me(&self);
}

pub struct PaintPartsSpinButtonCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    event_box: gtk::EventBox,
    entry: gtk::SpinButton,
    label: gtk::Label,
    menu: gtk::Menu,
    paint: Paint<C>,
    parts_changed_callbacks: RefCell<Vec<Box<Fn(u32)>>>,
    remove_me_callbacks: RefCell<Vec<Box<Fn(&PaintPartsSpinButton<C>)>>>
}

impl<C> PartialEq for PaintPartsSpinButtonCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn eq(&self, other: &PaintPartsSpinButtonCore<C>) -> bool {
        self.paint == other.paint
    }
}

pub type PaintPartsSpinButton<C> = Rc<PaintPartsSpinButtonCore<C>>;

impl<C> PackableWidgetInterface for PaintPartsSpinButton<C>
    where   C: Hash + Clone + PartialEq + Copy + Copy
{
    type PackableWidgetType = gtk::EventBox;

    fn pwo(&self) -> gtk::EventBox {
        self.event_box.clone()
    }
}

impl<C> PaintPartsSpinButtonInterface<C> for PaintPartsSpinButton<C>
    where   C: Hash + Clone + PartialEq + Copy + 'static
{
    type PaintPartsSpinButtonType = PaintPartsSpinButton<C>;

    fn create_with(paint: &Paint<C>, sensitive:bool) -> PaintPartsSpinButton<C> {
        let adj = gtk::Adjustment::new(0.0, 0.0, 999.0, 1.0, 10.0, 0.0);
        let label_text = paint.name();
        let parts_changed_callbacks: RefCell<Vec<Box<Fn(u32)>>> = RefCell::new(Vec::new());
        let remove_me_callbacks: RefCell<Vec<Box<Fn(&PaintPartsSpinButton<C>)>>> = RefCell::new(Vec::new());
        let spin_button = Rc::new(
            PaintPartsSpinButtonCore::<C> {
                event_box: gtk::EventBox::new(),
                entry: gtk::SpinButton::new(Some(&adj), 0.0, 0),
                label: gtk::Label::new(Some(label_text.as_str())),
                menu: gtk::Menu::new(),
                paint: paint.clone(),
                parts_changed_callbacks: parts_changed_callbacks,
                remove_me_callbacks: remove_me_callbacks
            }
        );
        let events = gdk::BUTTON_PRESS_MASK | gdk::BUTTON_RELEASE_MASK;
        spin_button.event_box.set_tooltip_text(Some(paint.tooltip_text().as_str()));
        spin_button.event_box.add_events(events.bits() as i32);
        spin_button.label.set_widget_colour(&paint.colour());
        spin_button.entry.set_widget_colour(&paint.colour());
        spin_button.entry.set_numeric(true);
        spin_button.entry.set_adjustment(&adj);
        spin_button.entry.set_sensitive(sensitive);
        // Build menu
        let remove_me_item = gtk::MenuItem::new_with_label("Remove Me");
        let spin_button_c = spin_button.clone();
        remove_me_item.connect_activate(
            move |_| {println!("Remove: {:?}", spin_button_c.paint.name())}
        );
        spin_button.menu.append(&remove_me_item.clone());
        spin_button.menu.show_all();
        //
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        hbox.pack_start(&spin_button.label.clone(), true, true, 0);
        hbox.pack_start(&spin_button.entry.clone(), false, false, 0);
        spin_button.event_box.add(&hbox);
        let spin_button_c = spin_button.clone();
        spin_button.entry.connect_value_changed(
            move |_| spin_button_c.inform_parts_changed()
        );
        let spin_button_c = spin_button.clone();
        spin_button.event_box.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        spin_button_c.menu.popup_at_pointer(None);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );
        spin_button
        // TODO: add "display paint" to PaintPartsSpinButton<C> popup menu
    }

    fn get_parts(&self) -> u32 {
        self.entry.get_value_as_int() as u32
    }

    fn set_parts(&self, parts: u32) {
        self.entry.set_value(parts as f64)
    }

    fn divide_parts(&self, divisor: u32) {
        let parts = self.entry.get_value_as_int() as u32 / divisor;
        self.entry.set_value(parts as f64);
    }

    fn get_colour_component(&self) -> ColourComponent {
        ColourComponent{
            colour: self.paint.colour().clone(),
            parts: self.entry.get_value_as_int() as u32
        }
    }

    fn get_paint_component(&self) -> PaintComponent<C> {
        PaintComponent::<C> {
            paint: self.paint.clone(),
            parts: self.entry.get_value_as_int() as u32
        }
    }

    fn set_sensitive(&self, sensitive: bool) {
        self.entry.set_sensitive(sensitive)
    }

    fn connect_parts_changed<F: 'static + Fn(u32)>(&self, callback: F) {
        self.parts_changed_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_parts_changed(&self) {
        let parts = self.entry.get_value_as_int() as u32;
        for callback in self.parts_changed_callbacks.borrow().iter() {
            callback(parts);
        }
    }

    fn connect_remove_me<F: 'static + Fn(&PaintPartsSpinButton<C>)>(&self, callback: F) {
        self.remove_me_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_remove_me(&self) {
        let spin_button = self.clone();
        for callback in self.remove_me_callbacks.borrow().iter() {
            callback(&spin_button);
        }
    }
}

pub trait PaintComponentsInterface<C>: PackableWidgetInterface
    where   C: Hash + Clone + PartialEq + Copy
{
    type PaintComponentsType;

    fn create_with(n_cols: u32, sensitive:bool) -> Self::PaintComponentsType;
    fn add_paint(&self, paint: &Paint<C>);
}

pub struct PaintComponentsBoxCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    vbox: gtk::Box,
    spin_buttons: RefCell<Vec<PaintPartsSpinButton<C>>>,
    h_boxes: RefCell<Vec<gtk::Box>>,
    count: Cell<u32>,
    n_cols: Cell<u32>,
    is_sensitive: Cell<bool>,
    supress_change_notification: Cell<bool>,
    colour_changed_callbacks: RefCell<Vec<Box<Fn(Option<&Colour>)>>>,
    paint_removed_callbacks: RefCell<Vec<Box<Fn(&Paint<C>)>>>
}

impl<C> PaintComponentsBoxCore<C>
    where   C: Hash + Clone + PartialEq + Copy + 'static
{
    pub fn set_sensitive(&self, sensitive: bool) {
        self.is_sensitive.set(sensitive);
        for spin_button in self.spin_buttons.borrow().iter() {
            spin_button.set_sensitive(sensitive)
        }
    }

    pub fn connect_colour_changed<F: 'static>(&self, callback: F)
        where for<'r> F: (Fn(Option<&'r Colour>))
    {
        self.colour_changed_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_colour_changed(&self) {
        if self.supress_change_notification.get() {
            return
        }
        let mut colour_mixer = colour_mix::ColourMixer::new();
        for spin_button in self.spin_buttons.borrow().iter() {
            let colour_component = spin_button.get_colour_component();
            colour_mixer.add(&colour_component)
        };
        if let Some(colour) = colour_mixer.get_colour() {
            for callback in self.colour_changed_callbacks.borrow().iter() {
                callback(Some(&colour));
            }
        } else {
            for callback in self.colour_changed_callbacks.borrow().iter() {
                callback(None);
            }
        }
    }

    pub fn connect_paint_removed<F: 'static>(&self, callback: F)
        where for<'r> F: (Fn(&'r Paint<C>))
    {
        self.paint_removed_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_paint_removed(&self, paint: &Paint<C>) {
        for callback in self.paint_removed_callbacks.borrow().iter() {
            callback(paint);
        }
    }

    fn pack_append(&self, spin_button: &PaintPartsSpinButton<C>) {
        if self.count.get() % self.n_cols.get() == 0 {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            self.h_boxes.borrow_mut().push(hbox.clone());
            self.vbox.pack_start(&hbox.clone(), true, true, 0);
        };
        let last_index = self.h_boxes.borrow().len() - 1;
        self.h_boxes.borrow()[last_index].pack_start(&spin_button.pwo(), true, true, 0);
        self.count.set(self.count.get() + 1)
    }

    fn unpack_all(&self) {
        for hbox in self.h_boxes.borrow().iter() {
            for child in hbox.get_children() {
                hbox.remove(&child)
            }
            self.vbox.remove(hbox)
        }
        self.h_boxes.borrow_mut().clear();
        self.count.set(0);
    }

    fn remove_spin_button(&self, spin_button: &PaintPartsSpinButton<C>) {
        self.unpack_all();
        let mut index: usize = 0;
        let mut spin_buttons = self.spin_buttons.borrow_mut();
        for (i, sb) in spin_buttons.iter().enumerate() {
            if sb == spin_button {
                index = i;
            } else {
                self.pack_append(sb);
            }
        };
        spin_buttons.remove(index);
        self.inform_paint_removed(&spin_button.paint);
        if spin_button.get_parts() > 0 {
            self.inform_colour_changed()
        }
    }
}

pub type PaintComponentsBox<C> = Rc<PaintComponentsBoxCore<C>>;

impl<C> PackableWidgetInterface for PaintComponentsBox<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    type PackableWidgetType = gtk::Box;

    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }
}

impl<C> PaintComponentsInterface<C> for PaintComponentsBox<C>
    where   C: Hash + Clone + PartialEq + Copy + 'static
{
    type PaintComponentsType = PaintComponentsBox<C>;

    fn create_with(n_cols: u32, sensitive:bool) -> PaintComponentsBox<C> {
        let pcb_core = PaintComponentsBoxCore::<C> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
            spin_buttons: RefCell::new(Vec::new()),
            h_boxes: RefCell::new(Vec::new()),
            count: Cell::new(0),
            n_cols: Cell::new(n_cols),
            is_sensitive: Cell::new(sensitive),
            supress_change_notification: Cell::new(false),
            colour_changed_callbacks: RefCell::new(Vec::new()),
            paint_removed_callbacks: RefCell::new(Vec::new()),
        };
        Rc::new(pcb_core)
    }

    fn add_paint(&self, paint: &Paint<C>) {
        for spin_button in self.spin_buttons.borrow().iter() {
            if spin_button.paint == *paint {
                return
            }
        }
        let pc = paint.clone();
        let spin_button = PaintPartsSpinButton::<C>::create_with(&pc, self.is_sensitive.get());
        let spin_button_c = spin_button.clone();
        self.spin_buttons.borrow_mut().push(spin_button_c);
        let self_c = self.clone();
        spin_button.connect_parts_changed(
            move |_| {self_c.inform_colour_changed()}
        );
        let self_c = self.clone();
        spin_button.connect_remove_me(
            move |sb| {self_c.remove_spin_button(sb)}
        );
        self.pack_append(&spin_button);
        self.vbox.show_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use paint::model_paint::*;

    #[test]
    fn paint_contributions_model_paint_box() {
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }

        let components_box = PaintComponentsBox::<ModelPaintCharacteristics>::create_with(6, true);
        let series = create_ideal_model_paint_series();
        for pair in [
            ("Red", RED),
            ("Green", GREEN),
            ("Blue", BLUE),
            ("Cyan", CYAN),
            ("Magenta", MAGENTA),
            ("Yellow", YELLOW),
            ("Black", BLACK),
            ("White", WHITE)
        ].iter()
        {
            assert_eq!(series.get_series_paint(pair.0).unwrap().colour().rgb(), pair.1);
            assert_eq!(series.get_paint(pair.0).unwrap().colour().rgb(), pair.1);
            let paint = series.get_paint(pair.0).unwrap();
            assert_eq!(paint.colour().rgb(), pair.1);
            components_box.add_paint(&paint);
        }
    }
}
