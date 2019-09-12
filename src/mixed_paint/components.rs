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
use std::rc::Rc;

use num::Integer;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::menu::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use dialogue::PaintDisplayWithCurrentTarget;

pub trait PaintPartsSpinButtonInterface<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn create_with(
        paint: &P,
        current_target: Option<&Colour>,
        sensitive: bool,
    ) -> PaintPartsSpinButton<A, C, P, D>;
    fn inform_remove_me(&self);
}

pub struct PaintPartsSpinButtonCore<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    event_box: gtk::EventBox,
    entry: gtk::SpinButton,
    label: gtk::Label,
    popup_menu: WrappedMenu,
    paint: P,
    current_target: RefCell<Option<Colour>>,
    dialog: RefCell<Option<D>>,
    parts_changed_callbacks: RefCell<Vec<Box<dyn Fn(u32)>>>,
    remove_me_callbacks: RefCell<Vec<Box<dyn Fn(&PaintPartsSpinButton<A, C, P, D>)>>>,
}

impl<A, C, P, D> PartialEq for PaintPartsSpinButtonCore<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn eq(&self, other: &PaintPartsSpinButtonCore<A, C, P, D>) -> bool {
        self.paint == other.paint
    }
}

impl_widget_wrapper!(event_box: gtk::EventBox, PaintPartsSpinButtonCore<A, C, P, D>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
            P: BasicPaintInterface<C> + 'static,
            D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
);

pub type PaintPartsSpinButton<A, C, P, D> = Rc<PaintPartsSpinButtonCore<A, C, P, D>>;

impl<A, C, P, D> PaintPartsSpinButtonInterface<A, C, P, D> for PaintPartsSpinButton<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn create_with(
        paint: &P,
        current_target: Option<&Colour>,
        sensitive: bool,
    ) -> PaintPartsSpinButton<A, C, P, D> {
        let adj = gtk::Adjustment::new(0.0, 0.0, 999.0, 1.0, 10.0, 0.0);
        let label_text = paint.name();
        let parts_changed_callbacks: RefCell<Vec<Box<dyn Fn(u32)>>> = RefCell::new(Vec::new());
        let remove_me_callbacks: RefCell<Vec<Box<dyn Fn(&PaintPartsSpinButton<A, C, P, D>)>>> =
            RefCell::new(Vec::new());
        let spin_button = Rc::new(PaintPartsSpinButtonCore::<A, C, P, D> {
            event_box: gtk::EventBox::new(),
            entry: gtk::SpinButton::new(Some(&adj), 0.0, 0),
            label: gtk::Label::new(Some(label_text.as_str())),
            popup_menu: WrappedMenu::new(&vec![]),
            paint: paint.clone(),
            current_target: RefCell::new(None),
            dialog: RefCell::new(None),
            parts_changed_callbacks: parts_changed_callbacks,
            remove_me_callbacks: remove_me_callbacks,
        });
        let events = gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK;
        spin_button
            .event_box
            .set_tooltip_text(Some(paint.tooltip_text().as_str()));
        spin_button.event_box.add_events(events);
        spin_button.label.set_widget_colour(&paint.colour());
        spin_button.entry.set_numeric(true);
        spin_button.entry.set_adjustment(&adj);
        spin_button.entry.set_sensitive(sensitive);
        spin_button.set_current_target(current_target);
        // Build menu
        let spin_button_c = spin_button.clone();
        spin_button
            .popup_menu
            .append_item(
                "info",
                "Paint Information",
                "Display this paint's information",
            )
            .connect_activate(move |_| {
                let needs_dialog = spin_button_c.dialog.borrow().is_none();
                if needs_dialog {
                    let target_colour = spin_button_c.get_current_target();
                    let target = if let Some(ref colour) = target_colour {
                        Some(colour)
                    } else {
                        None
                    };
                    let dialog = D::create(&spin_button_c.paint, target, &spin_button_c, vec![]);
                    let spin_button_c_c = spin_button_c.clone();
                    dialog.connect_destroy(move |_| {
                        *spin_button_c_c.dialog.borrow_mut() = None;
                    });
                    *spin_button_c.dialog.borrow_mut() = Some(dialog);
                };
                if let Some(ref dialog) = *spin_button_c.dialog.borrow() {
                    dialog.present()
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            });

        let spin_button_c = spin_button.clone();
        spin_button
            .popup_menu
            .append_item("remove", "Remove Me", "Remove this paint from the palette")
            .connect_activate(move |_| {
                spin_button_c.inform_remove_me();
            });
        //
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        hbox.pack_start(&spin_button.label.clone(), true, true, 0);
        hbox.pack_start(&spin_button.entry.clone(), false, false, 0);
        hbox.set_widget_colour(&paint.colour());
        let frame = gtk::Frame::new(None);
        frame.set_widget_colour(&paint.colour());
        frame.add(&hbox);
        spin_button.event_box.add(&frame);
        let spin_button_c = spin_button.clone();
        spin_button
            .entry
            .connect_value_changed(move |_| spin_button_c.inform_parts_changed());
        let spin_button_c = spin_button.clone();
        spin_button
            .event_box
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        spin_button_c
                            .popup_menu
                            .set_sensitivities(spin_button_c.get_parts() == 0, &["remove"]);
                        spin_button_c.popup_menu.popup_at_event(event);
                        return Inhibit(true);
                    }
                }
                Inhibit(false)
            });

        spin_button
    }

    fn inform_remove_me(&self) {
        let spin_button = self.clone();
        for callback in self.remove_me_callbacks.borrow().iter() {
            callback(&spin_button);
        }
    }
}

impl<A, C, P, D> PaintPartsSpinButtonCore<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn paint(&self) -> P {
        self.paint.clone()
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

    fn get_paint_component(&self) -> (P, u32) {
        (self.paint.clone(), self.entry.get_value_as_int() as u32)
    }

    fn set_sensitive(&self, sensitive: bool) {
        self.entry.set_sensitive(sensitive)
    }

    fn connect_parts_changed<F: 'static + Fn(u32)>(&self, callback: F) {
        self.parts_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_parts_changed(&self) {
        let parts = self.entry.get_value_as_int() as u32;
        for callback in self.parts_changed_callbacks.borrow().iter() {
            callback(parts);
        }
    }

    fn connect_remove_me<F: 'static + Fn(&PaintPartsSpinButton<A, C, P, D>)>(&self, callback: F) {
        self.remove_me_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn close_dialog(&self) {
        if let Some(ref dialog) = *self.dialog.borrow() {
            dialog.close();
        }
    }

    fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if let Some(ref dialog) = *self.dialog.borrow() {
            dialog.set_current_target(new_current_target);
        }
        if let Some(target) = new_current_target {
            *self.current_target.borrow_mut() = Some(target.clone())
        } else {
            *self.current_target.borrow_mut() = None
        }
    }

    fn get_current_target(&self) -> Option<Colour> {
        if let Some(ref colour) = *self.current_target.borrow() {
            Some(colour.clone())
        } else {
            None
        }
    }
}

pub trait PaintComponentsBoxInterface<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn create_with(n_cols: u32, sensitive: bool) -> PaintComponentsBox<A, C, P, D>;
    fn add_paint(&self, paint: &P);
    fn iter_colour_components(&self) -> ColourPartsIter<A, C, P, D>;
}

pub struct PaintComponentsBoxCore<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    vbox: gtk::Box,
    spin_buttons: RefCell<Vec<PaintPartsSpinButton<A, C, P, D>>>,
    h_boxes: RefCell<Vec<gtk::Box>>,
    count: Cell<u32>,
    n_cols: Cell<u32>,
    is_sensitive: Cell<bool>,
    supress_change_notification: Cell<bool>,
    current_target: RefCell<Option<Colour>>,
    contributions_changed_callbacks: RefCell<Vec<Box<dyn Fn()>>>,
    removal_requested_callbacks: RefCell<Vec<Box<dyn Fn(&P)>>>,
}

impl_widget_wrapper!(vbox: gtk::Box, PaintComponentsBoxCore<A, C, P, D>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
            P: BasicPaintInterface<C> + 'static,
            D: PaintDisplayWithCurrentTarget<A, C, P> + 'static
);

impl<A, C, P, D> PaintComponentsBoxCore<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn find_paint_index(&self, paint: &P) -> Result<usize, usize> {
        let result = self
            .spin_buttons
            .borrow()
            .binary_search_by_key(paint, |spinner| spinner.paint());
        result
    }

    pub fn has_listeners(&self) -> bool {
        self.contributions_changed_callbacks.borrow().len() > 0
    }

    pub fn is_being_used(&self, paint: &P) -> bool {
        if let Ok(index) = self.find_paint_index(paint) {
            return self.spin_buttons.borrow()[index].get_parts() > 0;
        };
        false
    }

    pub fn set_sensitive(&self, sensitive: bool) {
        self.is_sensitive.set(sensitive);
        for spin_button in self.spin_buttons.borrow().iter() {
            spin_button.set_sensitive(sensitive)
        }
    }

    pub fn connect_contributions_changed<F: 'static>(&self, callback: F)
    where
        F: (Fn()),
    {
        self.contributions_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    fn inform_contributions_changed(&self) {
        for callback in self.contributions_changed_callbacks.borrow().iter() {
            callback();
        }
    }

    pub fn has_contributions(&self) -> bool {
        self.spin_buttons.borrow().iter().any(|s| s.get_parts() > 0)
    }

    pub fn connect_removal_requested<F: 'static>(&self, callback: F)
    where
        for<'r> F: (Fn(&'r P)),
    {
        self.removal_requested_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    pub fn inform_removal_requested(&self, paint: &P) {
        for callback in self.removal_requested_callbacks.borrow().iter() {
            callback(paint);
        }
    }

    fn pack_append(&self, spin_button: &PaintPartsSpinButton<A, C, P, D>) {
        if self.count.get() % self.n_cols.get() == 0 {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            self.h_boxes.borrow_mut().push(hbox.clone());
            self.vbox.pack_start(&hbox.clone(), false, false, 0);
        };
        let last_index = self.h_boxes.borrow().len() - 1;
        self.h_boxes.borrow()[last_index].pack_start(&spin_button.pwo(), true, true, 0);
        self.count.set(self.count.get() + 1);
    }

    fn repack_all(&self) {
        for hbox in self.h_boxes.borrow().iter() {
            for child in hbox.get_children() {
                hbox.remove(&child)
            }
            self.vbox.remove(hbox)
        }
        self.h_boxes.borrow_mut().clear();
        self.count.set(0);
        for spin_button in self.spin_buttons.borrow().iter() {
            self.pack_append(spin_button);
        }
        self.vbox.show_all();
    }

    pub fn remove_paint(&self, paint: &P) {
        let r_index = self.find_paint_index(paint);
        if let Ok(index) = r_index {
            let spin_button = self.spin_buttons.borrow_mut().remove(index);
            spin_button.close_dialog();
            self.repack_all();
            if spin_button.get_parts() > 0 {
                self.inform_contributions_changed()
            }
        }
    }

    pub fn remove_unused_spin_buttons(&self, in_use: &Vec<P>) -> Vec<P> {
        let mut keepers: Vec<PaintPartsSpinButton<A, C, P, D>> = vec![];
        let mut removed_paints: Vec<P> = vec![];
        for spin_button in self.spin_buttons.borrow_mut().iter() {
            if spin_button.get_parts() > 0 || in_use.binary_search(&spin_button.paint).is_ok() {
                keepers.push(spin_button.clone());
            } else {
                spin_button.close_dialog();
                removed_paints.push(spin_button.paint());
            }
        }
        *self.spin_buttons.borrow_mut() = keepers;
        self.repack_all();
        removed_paints
    }

    pub fn reset_all_parts_to_zero(&self) {
        self.supress_change_notification.set(true);
        for spin_button in self.spin_buttons.borrow().iter() {
            spin_button.set_parts(0);
        }
        self.supress_change_notification.set(false);
        self.inform_contributions_changed();
    }

    pub fn get_gcd(&self) -> u32 {
        self.spin_buttons
            .borrow()
            .iter()
            .fold(0, |gcd, s| gcd.gcd(&s.get_parts()))
    }

    pub fn divide_all_parts_by(&self, gcd: u32) {
        if gcd > 1 {
            self.supress_change_notification.set(true);
            for spin_button in self.spin_buttons.borrow().iter() {
                spin_button.divide_parts(gcd);
            }
            self.supress_change_notification.set(false);
        }
    }

    pub fn get_paint_components(&self) -> Vec<(P, u32)> {
        self.spin_buttons
            .borrow()
            .iter()
            .filter(|s| s.get_parts() > 0)
            .map(|s| s.get_paint_component())
            .collect()
    }

    pub fn set_current_target(&self, new_current_target: Option<&Colour>) {
        for spin_button in self.spin_buttons.borrow().iter() {
            spin_button.set_current_target(new_current_target)
        }
        if let Some(target) = new_current_target {
            *self.current_target.borrow_mut() = Some(target.clone())
        } else {
            *self.current_target.borrow_mut() = None
        }
    }

    pub fn get_current_target(&self) -> Option<Colour> {
        if let Some(ref colour) = *self.current_target.borrow() {
            Some(colour.clone())
        } else {
            None
        }
    }
}

pub type PaintComponentsBox<A, C, P, D> = Rc<PaintComponentsBoxCore<A, C, P, D>>;

impl<A, C, P, D> PaintComponentsBoxInterface<A, C, P, D> for PaintComponentsBox<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    fn create_with(n_cols: u32, sensitive: bool) -> PaintComponentsBox<A, C, P, D> {
        let pcb_core = PaintComponentsBoxCore::<A, C, P, D> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
            spin_buttons: RefCell::new(Vec::new()),
            h_boxes: RefCell::new(Vec::new()),
            count: Cell::new(0),
            n_cols: Cell::new(n_cols),
            is_sensitive: Cell::new(sensitive),
            supress_change_notification: Cell::new(false),
            current_target: RefCell::new(None),
            contributions_changed_callbacks: RefCell::new(Vec::new()),
            removal_requested_callbacks: RefCell::new(Vec::new()),
        };
        Rc::new(pcb_core)
    }

    fn add_paint(&self, paint: &P) {
        if let Err(index) = self.find_paint_index(paint) {
            let pc = paint.clone();
            let target_colour = self.get_current_target();
            let target = if let Some(ref colour) = target_colour {
                Some(colour)
            } else {
                None
            };
            let spin_button = PaintPartsSpinButton::<A, C, P, D>::create_with(
                &pc,
                target,
                self.is_sensitive.get(),
            );
            let spin_button_c = spin_button.clone();
            self.spin_buttons.borrow_mut().insert(index, spin_button_c);
            let self_c = self.clone();
            spin_button.connect_parts_changed(move |_| self_c.inform_contributions_changed());
            let self_c = self.clone();
            spin_button.connect_remove_me(move |sb| self_c.inform_removal_requested(&sb.paint()));
            self.pack_append(&spin_button);
            self.vbox.show_all();
        }
    }

    fn iter_colour_components(&self) -> ColourPartsIter<A, C, P, D> {
        ColourPartsIter {
            components: self.clone(),
            index: 0,
        }
    }
}

pub struct ColourPartsIter<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    components: PaintComponentsBox<A, C, P, D>,
    index: usize,
}

impl<A, C, P, D> Iterator for ColourPartsIter<A, C, P, D>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
    P: BasicPaintInterface<C> + 'static,
    D: PaintDisplayWithCurrentTarget<A, C, P> + 'static,
{
    type Item = (Colour, u32);

    fn next(&mut self) -> Option<(Colour, u32)> {
        let spin_buttons = self.components.spin_buttons.borrow();
        while self.index < spin_buttons.len() {
            let spin_button = &spin_buttons[self.index];
            self.index += 1;
            let parts = spin_button.get_parts();
            if parts > 0 {
                return Some((spin_button.paint.colour(), parts));
            }
        }
        None
    }
}
