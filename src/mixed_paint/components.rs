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
use colour_mix::*;
use mixed_paint::display::*;
use series_paint::*;
use super::*;

enum DialogOption<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    Series(SeriesPaintDisplayDialog<A, C>),
    Mixed(MixedPaintDisplayDialog<A, C>),
    None
}

impl<A, C> DialogOption<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    pub fn is_none(&self) -> bool {
        match *self {
            DialogOption::None => true,
            _ => false,
        }
    }

    pub fn present(&self) {
        match *self {
            DialogOption::Series(ref dialog) => dialog.present(),
            DialogOption::Mixed(ref dialog) => dialog.present(),
            DialogOption::None => ()
        }
    }

    pub fn close(&self) {
        match *self {
            DialogOption::Series(ref dialog) => dialog.close(),
            DialogOption::Mixed(ref dialog) => dialog.close(),
            DialogOption::None => ()
        }
    }

    pub fn set_current_target(&self, new_current_target: Option<&Colour>) {
        match *self {
            DialogOption::Series(ref dialog) => dialog.set_current_target(new_current_target),
            DialogOption::Mixed(ref dialog) => dialog.set_current_target(new_current_target),
            DialogOption::None => ()
        }
    }
}

pub trait PaintPartsSpinButtonInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn create_with(paint: &Paint<C>, current_target: Option<&Colour>, sensitive:bool) -> PaintPartsSpinButton<A, C>;
    fn get_parts(&self) -> u32;
    fn set_parts(&self, parts: u32);
    fn divide_parts(&self, divisor: u32);
    fn get_colour_component(&self) -> ColourComponent;
    fn get_paint_component(&self) -> PaintComponent<C>;
    fn set_sensitive(&self, sensitive: bool);
    fn connect_parts_changed<F: 'static + Fn(u32)>(&self, callback: F);
    fn inform_parts_changed(&self);
    fn connect_remove_me<F: 'static + Fn(&PaintPartsSpinButton<A, C>)>(&self, callback: F);
    fn inform_remove_me(&self);
    fn close_dialog(&self);
    fn set_current_target(&self, new_current_target: Option<&Colour>);
    fn get_current_target(&self) -> Option<Colour>;
}

pub struct PaintPartsSpinButtonCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    event_box: gtk::EventBox,
    entry: gtk::SpinButton,
    label: gtk::Label,
    popup_menu: WrappedMenu,
    paint: Paint<C>,
    current_target: RefCell<Option<Colour>>,
    dialog: RefCell<DialogOption<A, C>>,
    parts_changed_callbacks: RefCell<Vec<Box<Fn(u32)>>>,
    remove_me_callbacks: RefCell<Vec<Box<Fn(&PaintPartsSpinButton<A, C>)>>>
}

impl<A, C> PartialEq for PaintPartsSpinButtonCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn eq(&self, other: &PaintPartsSpinButtonCore<A, C>) -> bool {
        self.paint == other.paint
    }
}

impl<A, C> WidgetWrapper for PaintPartsSpinButtonCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    type PWT = gtk::EventBox;

    fn pwo(&self) -> gtk::EventBox {
        self.event_box.clone()
    }
}

pub type PaintPartsSpinButton<A, C> = Rc<PaintPartsSpinButtonCore<A, C>>;

impl<A, C> PaintPartsSpinButtonInterface<A, C> for PaintPartsSpinButton<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn create_with(paint: &Paint<C>, current_target: Option<&Colour>, sensitive:bool) -> PaintPartsSpinButton<A, C> {
        let adj = gtk::Adjustment::new(0.0, 0.0, 999.0, 1.0, 10.0, 0.0);
        let label_text = paint.name();
        let parts_changed_callbacks: RefCell<Vec<Box<Fn(u32)>>> = RefCell::new(Vec::new());
        let remove_me_callbacks: RefCell<Vec<Box<Fn(&PaintPartsSpinButton<A, C>)>>> = RefCell::new(Vec::new());
        let spin_button = Rc::new(
            PaintPartsSpinButtonCore::<A, C> {
                event_box: gtk::EventBox::new(),
                entry: gtk::SpinButton::new(Some(&adj), 0.0, 0),
                label: gtk::Label::new(Some(label_text.as_str())),
                popup_menu: WrappedMenu::new(&vec![]),
                paint: paint.clone(),
                current_target: RefCell::new(None),
                dialog: RefCell::new(DialogOption::None),
                parts_changed_callbacks: parts_changed_callbacks,
                remove_me_callbacks: remove_me_callbacks
            }
        );
        let events = gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK;
        spin_button.event_box.set_tooltip_text(Some(paint.tooltip_text().as_str()));
        spin_button.event_box.add_events(events.bits() as i32);
        spin_button.label.set_widget_colour(&paint.colour());
        spin_button.entry.set_numeric(true);
        spin_button.entry.set_adjustment(&adj);
        spin_button.entry.set_sensitive(sensitive);
        spin_button.set_current_target(current_target);
        // Build menu
        let spin_button_c = spin_button.clone();
        spin_button.popup_menu.append_item(
            "info",
            "Paint Information",
            "Display this paint's information",
        ).connect_activate(
            move |_| {
                let mut o_dialog = spin_button_c.dialog.borrow_mut();
                if o_dialog.is_none() {
                    let target_colour = spin_button_c.get_current_target();
                    let target = if let Some(ref colour) = target_colour {
                        Some(colour)
                    } else {
                        None
                    };
                    match spin_button_c.paint {
                        Paint::Series(ref paint) => {
                            let dialog = SeriesPaintDisplayDialog::<A, C>::create(paint, target, None, vec![]);
                            dialog.set_transient_for_from(&spin_button_c.pwo());
                            let spin_button_c_c = spin_button_c.clone();
                            dialog.connect_destroy(
                                move |_| { *spin_button_c_c.dialog.borrow_mut() = DialogOption::None; }
                            );
                            *o_dialog = DialogOption::Series(dialog.clone());
                            dialog.show();
                        },
                        Paint::Mixed(ref paint) => {
                            let dialog = MixedPaintDisplayDialog::<A, C>::create(paint, target, None, vec![]);
                            dialog.set_transient_for_from(&spin_button_c.pwo());
                            let spin_button_c_c = spin_button_c.clone();
                            dialog.connect_destroy(
                                move |_| { *spin_button_c_c.dialog.borrow_mut() = DialogOption::None; }
                            );
                            *o_dialog = DialogOption::Mixed(dialog.clone());
                            dialog.show();
                        },
                    }
                } else {
                    o_dialog.present();
                }
            }
        );

        let spin_button_c = spin_button.clone();
        spin_button.popup_menu.append_item(
            "remove",
            "Remove Me",
            "Remove this paint from the palette",
        ).connect_activate(
            move |_| { spin_button_c.inform_remove_me(); }
        );
        //
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        hbox.pack_start(&spin_button.label.clone(), true, true, 0);
        hbox.pack_start(&spin_button.entry.clone(), false, false, 0);
        let frame = gtk::Frame::new(None);
        frame.add(&hbox);
        spin_button.event_box.add(&frame);
        let spin_button_c = spin_button.clone();
        spin_button.entry.connect_value_changed(
            move |_| spin_button_c.inform_parts_changed()
        );
        let spin_button_c = spin_button.clone();
        spin_button.event_box.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        spin_button_c.popup_menu.popup_at_event(event);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );

        spin_button
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

    fn connect_remove_me<F: 'static + Fn(&PaintPartsSpinButton<A, C>)>(&self, callback: F) {
        self.remove_me_callbacks.borrow_mut().push(Box::new(callback))
    }

    fn close_dialog(&self) {
        self.dialog.borrow().close();
    }

    fn inform_remove_me(&self) {
        let spin_button = self.clone();
        for callback in self.remove_me_callbacks.borrow().iter() {
            callback(&spin_button);
        }
    }

    fn set_current_target(&self, new_current_target: Option<&Colour>) {
        self.dialog.borrow().set_current_target(new_current_target);
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

pub trait PaintComponentsBoxInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn create_with(n_cols: u32, sensitive:bool) -> PaintComponentsBox<A, C>;
    fn add_paint(&self, paint: &Paint<C>);
    fn add_series_paint(&self, paint: &SeriesPaint<C>);
}

pub struct PaintComponentsBoxCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    vbox: gtk::Box,
    spin_buttons: RefCell<Vec<PaintPartsSpinButton<A, C>>>,
    h_boxes: RefCell<Vec<gtk::Box>>,
    count: Cell<u32>,
    n_cols: Cell<u32>,
    is_sensitive: Cell<bool>,
    supress_change_notification: Cell<bool>,
    current_target: RefCell<Option<Colour>>,
    colour_changed_callbacks: RefCell<Vec<Box<Fn(Option<&Colour>)>>>,
    paint_removed_callbacks: RefCell<Vec<Box<Fn(&Paint<C>)>>>
}

impl<A, C> WidgetWrapper for PaintComponentsBoxCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    type PWT = gtk::Box;

    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }
}

impl<A, C> PaintComponentsBoxCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
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
        let mut colour_mixer = ColourMixer::new();
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

    pub fn has_colour(&self) -> bool {
        for spin_button in self.spin_buttons.borrow().iter() {
            if spin_button.get_parts() > 0 {
                return true
            }
        };
        false
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

    fn pack_append(&self, spin_button: &PaintPartsSpinButton<A, C>) {
        if self.count.get() % self.n_cols.get() == 0 {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            self.h_boxes.borrow_mut().push(hbox.clone());
            self.vbox.pack_start(&hbox.clone(), false, false, 0);
        };
        let last_index = self.h_boxes.borrow().len() - 1;
        self.h_boxes.borrow()[last_index].pack_start(&spin_button.pwo(), true, true, 0);
        self.count.set(self.count.get() + 1);
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

    fn remove_spin_button(&self, spin_button: &PaintPartsSpinButton<A, C>) {
        self.unpack_all();
        spin_button.close_dialog();
        let colour_will_change = spin_button.get_parts() > 0;
        { // NB: needed to avoid mutable borrow conflict
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
            self.vbox.show_all();
        }
        self.inform_paint_removed(&spin_button.paint);
        if colour_will_change {
            self.inform_colour_changed()
        }
    }

    pub fn remove_unused_spin_buttons(&self) {
        let mut unused: Vec<PaintPartsSpinButton<A, C>> = vec![];
        for spin_button in self.spin_buttons.borrow().iter() {
            if spin_button.get_parts() == 0 {
                unused.push(spin_button.clone())
            }
        }
        for spin_button in unused.iter() {
            self.remove_spin_button(spin_button)
        }
    }

    pub fn reset_all_parts_to_zero(&self) {
        self.supress_change_notification.set(true);
        for spin_button in self.spin_buttons.borrow().iter() {
            spin_button.set_parts(0);
        }
        self.supress_change_notification.set(false);
        self.inform_colour_changed();
    }

    pub fn simplify_parts(&self) {
        let mut gcd: u32 = 0;
        for spin_button in self.spin_buttons.borrow().iter() {
            gcd = gcd.gcd(&spin_button.get_parts());
        }
        if gcd > 1 {
            self.supress_change_notification.set(true);
            for spin_button in self.spin_buttons.borrow().iter() {
                spin_button.divide_parts(gcd);
            }
            self.supress_change_notification.set(false);
        }
    }

    pub fn get_paint_components(&self) -> Vec<PaintComponent<C>> {
        let mut components = vec![];
        for spin_button in self.spin_buttons.borrow().iter() {
            if spin_button.get_parts() > 0 {
                components.push(spin_button.get_paint_component())
            }
        }
        components
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

pub type PaintComponentsBox<A, C> = Rc<PaintComponentsBoxCore<A, C>>;

impl<A, C> PaintComponentsBoxInterface<A, C> for PaintComponentsBox<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static,
{
    fn create_with(n_cols: u32, sensitive:bool) -> PaintComponentsBox<A, C> {
        let pcb_core = PaintComponentsBoxCore::<A, C> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
            spin_buttons: RefCell::new(Vec::new()),
            h_boxes: RefCell::new(Vec::new()),
            count: Cell::new(0),
            n_cols: Cell::new(n_cols),
            is_sensitive: Cell::new(sensitive),
            supress_change_notification: Cell::new(false),
            current_target: RefCell::new(None),
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
        let target_colour = self.get_current_target();
        let target = if let Some(ref colour) = target_colour {
            Some(colour)
        } else {
            None
        };
        let spin_button = PaintPartsSpinButton::<A, C>::create_with(&pc, target, self.is_sensitive.get());
        let spin_button_c = spin_button.clone();
        self.spin_buttons.borrow_mut().push(spin_button_c);
        let self_c = self.clone();
        spin_button.connect_parts_changed(
            move |_| {self_c.inform_colour_changed()}
        );
        let self_c = self.clone();
        spin_button.connect_remove_me(
            move |sb| { self_c.remove_spin_button(sb) }
        );
        self.pack_append(&spin_button);
        self.vbox.show_all();
    }

    fn add_series_paint(&self, paint: &SeriesPaint<C>) {
        self.add_paint(&Paint::Series(paint.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::fmt;

    use pw_gix::rgb_math::rgb::*;

    use error::*;

    #[derive(Debug, PartialEq, Hash, Clone, Copy)]
    pub struct EPC { }
    pub struct Entry { }

    impl fmt::Display for EPC {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "nothing")
        }
    }

    impl CharacteristicsEntryInterface<EPC> for Rc<Entry> {
        fn create() -> Self { Rc::new(Entry{})}
        fn pwo(&self) -> gtk::Grid { gtk::Grid::new() }
        fn get_characteristics(&self) -> Option<EPC> { None }
        fn set_characteristics(&self, _o_characteristics: Option<&EPC>) {}
        fn connect_changed<F: 'static + Fn()>(&self, _callback: F) {}
    }

    impl CharacteristicsInterface for EPC {
        type Entry = Rc<Entry>;

        fn tv_row_len() -> usize {0}

        fn tv_columns(_start_col_id: i32) -> Vec<gtk::TreeViewColumn> {
            vec![]
        }

        fn gui_display_widget(&self) -> gtk::Box {
            gtk::Box::new(gtk::Orientation::Vertical, 1)
        }

        fn from_floats(_floats: &Vec<f64>) -> EPC {
            EPC{}
        }

        fn to_floats(&self) -> Vec<f64> {
            vec![]
        }

        fn tv_rows(&self) -> Vec<gtk::Value> {
            vec![]
        }
    }

    impl FromStr for EPC {
        type Err = PaintError;

        fn from_str(string: &str) -> Result<EPC, PaintError> {
            if string.len() == 0 {
                 Ok(EPC{})
            } else {
                 Ok(EPC{})
            }
        }
    }

    #[test]
    fn paint_contributions_model_paint_box() {
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }

        let components_box = PaintComponentsBox::<EPC>::create_with(6, true);
        let series = PaintSeries::<EPC>::create("empty", "empty");
        let epc = EPC::from_str(&"").unwrap();
        for spec in [
            BasicPaintSpec::<EPC>{rgb: RED, name: "Red".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: GREEN, name: "Green".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: BLUE, name: "Blue".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: CYAN, name: "Cyan".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: MAGENTA, name: "Magenta".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: YELLOW, name: "Yellow".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: BLACK, name: "Black".to_string(), notes: "".to_string(), characteristics: epc},
            BasicPaintSpec::<EPC>{rgb: WHITE, name: "White".to_string(), notes: "".to_string(), characteristics: epc},
        ].iter()
        {
            if let Err(err) = series.add_paint(spec) {
                panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
            }
        }

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
            match series.get_paint(pair.0) {
                Some(paint) => components_box.add_paint(&paint),
                None => panic!("File: {:?} Line: {:?}", file!(), line!())
            };
        }
    }
}
