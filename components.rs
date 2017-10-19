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
}

pub struct PaintPartsSpinButtonCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    event_box: gtk::EventBox,
    entry: gtk::SpinButton,
    label: gtk::Label,
    paint: Paint<C>,
    callbacks: RefCell<Vec<Box<Fn(u32)>>>
}

pub type PaintPartsSpinButton<C> = Rc<PaintPartsSpinButtonCore<C>>;

impl<C> PackableWidgetInterface for PaintPartsSpinButton<C>
    where   C: Hash + Clone + PartialEq + Copy
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
        let callbacks: RefCell<Vec<Box<Fn(u32)>>> = RefCell::new(Vec::new());
        let spin_button = Rc::new(
            PaintPartsSpinButtonCore::<C> {
                event_box: gtk::EventBox::new(),
                entry: gtk::SpinButton::new(Some(&adj), 0.0, 0),
                label: gtk::Label::new(Some(label_text.as_str())),
                paint: paint.clone(),
                callbacks: callbacks
            }
        );
        let events = gdk::BUTTON_PRESS_MASK | gdk::BUTTON_RELEASE_MASK;
        spin_button.event_box.add_events(events.bits() as i32);
        spin_button.label.set_widget_colour(&paint.colour());
        spin_button.entry.set_widget_colour(&paint.colour());
        spin_button.entry.set_numeric(true);
        spin_button.entry.set_adjustment(&adj);
        spin_button.entry.set_sensitive(sensitive);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        hbox.pack_start(&spin_button.label.clone(), true, true, 0);
        hbox.pack_start(&spin_button.entry.clone(), false, false, 0);
        spin_button.event_box.add(&hbox);
        let spin_button_c = spin_button.clone();
        spin_button.entry.connect_value_changed(
            move |_| spin_button_c.inform_parts_changed()
        );
        spin_button
        // TODO: add popup menu to PaintPartsSpinButton<C>
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
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_parts_changed(&self) {
        let parts = self.entry.get_value_as_int() as u32;
        for callback in self.callbacks.borrow().iter() {
            callback(parts);
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
    callbacks: RefCell<Vec<Box<Fn(Option<&Colour>)>>>
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
        self.callbacks.borrow_mut().push(Box::new(callback))
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
            println!("NEW COLOUR: {:?}", colour);
            for callback in self.callbacks.borrow().iter() {
                callback(Some(&colour));
            }
        } else {
            println!("NEW COLOUR: None");
            for callback in self.callbacks.borrow().iter() {
                callback(None);
            }
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
            callbacks: RefCell::new(Vec::new())
        };
        Rc::new(pcb_core)
    }

    fn add_paint(&self, paint: &Paint<C>) {
        let pc = paint.clone();
        let spin_button = PaintPartsSpinButton::<C>::create_with(&pc, self.is_sensitive.get());
        let self_c = self.clone();
        spin_button.connect_parts_changed(
            move |_| {self_c.inform_colour_changed()}
        );
        self.pack_append(&spin_button);
        self.vbox.show_all();
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
