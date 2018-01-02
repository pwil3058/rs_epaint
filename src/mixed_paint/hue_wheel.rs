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

use std;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;
use pw_gix::colour::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use display::*;
use graticule::*;
use series_paint::*;
use shape::*;
use super::*;
use super::target::*;

// CHOSEN_ITEM
#[derive(Debug)]
pub enum ChosenItem<C: CharacteristicsInterface> {
    SeriesPaint(SeriesPaint<C>),
    MixedPaint(MixedPaint<C>),
    TargetColour(TargetColour),
    None
}

impl<C: CharacteristicsInterface> ChosenItem<C> {
    pub fn is_series_paint(&self) -> bool {
        match *self {
            ChosenItem::SeriesPaint(_) => true,
            _ => false
        }
    }
    pub fn is_mixed_paint(&self) -> bool {
        match *self {
            ChosenItem::MixedPaint(_) => true,
            _ => false
        }
    }

    pub fn is_target_colour(&self) -> bool {
        match *self {
            ChosenItem::TargetColour(_) => true,
            _ => false
        }
    }

    pub fn is_none(&self) -> bool {
        match *self {
            ChosenItem::None => true,
            _ => false
        }
    }
}

// WHEEL
pub struct MixerHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    menu: gtk::Menu,
    paint_info_item: gtk::MenuItem,
    add_paint_item: gtk::MenuItem,
    series_paints: SeriesPaintShapeList<C>,
    mixed_paints: MixedPaintShapeList<C>,
    target_colours: TargetColourShapeList,
    chosen_item: RefCell<ChosenItem<C>>,
    graticule: Graticule,
    add_series_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    add_mixed_paint_callbacks: RefCell<Vec<Box<Fn(&MixedPaint<C>)>>>,
    series_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
    mixed_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
}

impl<A, C> WidgetWrapper<gtk::DrawingArea> for MixerHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn pwo(&self) -> gtk::DrawingArea {
        self.graticule.drawing_area()
    }
}

pub type MixerHueAttrWheel<A, C> = Rc<MixerHueAttrWheelCore<A, C>>;

pub trait MixerHueAttrWheelInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn create(attr: ScalarAttribute) -> MixerHueAttrWheel<A, C>;
}

impl<A, C> MixerHueAttrWheelInterface<A, C> for MixerHueAttrWheel<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn create(attr: ScalarAttribute) -> MixerHueAttrWheel<A, C> {
        let menu = gtk::Menu::new();
        let paint_info_item = gtk::MenuItem::new_with_label("Information");
        menu.append(&paint_info_item.clone());
        let add_paint_item = gtk::MenuItem::new_with_label("Add to Mixer");
        add_paint_item.set_visible(false);
        menu.append(&add_paint_item.clone());
        menu.show_all();
        let wheel = Rc::new(
            MixerHueAttrWheelCore::<A, C> {
                menu: menu,
                paint_info_item: paint_info_item,
                add_paint_item: add_paint_item,
                series_paints: SeriesPaintShapeList::<C>::new(attr),
                mixed_paints: MixedPaintShapeList::<C>::new(attr),
                target_colours: TargetColourShapeList::new(attr),
                graticule: Graticule::create(attr),
                chosen_item: RefCell::new(ChosenItem::None),
                add_series_paint_callbacks: RefCell::new(Vec::new()),
                add_mixed_paint_callbacks: RefCell::new(Vec::new()),
                series_paint_dialogs: RefCell::new(HashMap::new()),
                mixed_paint_dialogs: RefCell::new(HashMap::new()),
            }
        );
        let wheel_c = wheel.clone();
        wheel.graticule.drawing_area().connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let chosen_item = wheel_c.get_item_at(Point::from(event.get_position()));
                        wheel_c.paint_info_item.set_sensitive(!chosen_item.is_none());
                        if chosen_item.is_series_paint() {
                            wheel_c.add_paint_item.set_sensitive(true);
                        } else if chosen_item.is_mixed_paint() {
                            wheel_c.add_paint_item.set_sensitive(wheel_c.add_mixed_paint_callbacks.borrow().len() > 0);
                        } else {
                            wheel_c.add_paint_item.set_sensitive(false);
                        };
                        *wheel_c.chosen_item.borrow_mut() = chosen_item;
                        // TODO: needs v3_22: wheel_c.menu.popup_at_pointer(None);
                        wheel_c.menu.popup_easy(event.get_button(), event.get_time());
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );
        let wheel_c = wheel.clone();
        wheel.paint_info_item.connect_activate(
            move |_| {
                let target_colour = wheel_c.graticule.current_target_colour().clone();
                let target = if let Some(ref colour) = target_colour {
                    Some(colour)
                } else {
                    None
                };
                match *wheel_c.chosen_item.borrow() {
                    ChosenItem::SeriesPaint(ref paint) => {
                        let have_listeners = wheel_c.add_series_paint_callbacks.borrow().len() > 0;
                        if have_listeners {
                            let wheel_c_c = wheel_c.clone();
                            let paint_c = paint.clone();
                            let spec = PaintDisplayButtonSpec {
                                label: "Add".to_string(),
                                tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                                callback:  Box::new(move || wheel_c_c.inform_add_series_paint(&paint_c))
                            };
                            let dialog = PaintDisplayDialog::<A, C>::series_create(&paint, target, None, vec![spec]);
                            dialog.set_transient_for_from(&wheel_c.pwo());
                            let wheel_c_c = wheel_c.clone();
                            dialog.connect_destroy(
                                move |id| { wheel_c_c.series_paint_dialogs.borrow_mut().remove(&id); }
                            );
                            wheel_c.series_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        } else {
                            PaintDisplayDialog::<A, C>::series_create(&paint, None, None, vec![]).show();
                        }
                    },
                    ChosenItem::MixedPaint(ref paint) => {
                        let have_listeners = wheel_c.add_mixed_paint_callbacks.borrow().len() > 0;
                        if have_listeners {
                            let wheel_c_c = wheel_c.clone();
                            let paint_c = paint.clone();
                            let spec = PaintDisplayButtonSpec {
                                label: "Add".to_string(),
                                tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                                callback:  Box::new(move || wheel_c_c.inform_add_mixed_paint(&paint_c))
                            };
                            let dialog = PaintDisplayDialog::<A, C>::mixed_create(&paint, target, None, vec![spec]);
                            dialog.set_transient_for_from(&wheel_c.pwo());
                            let wheel_c_c = wheel_c.clone();
                            dialog.connect_destroy(
                                move |id| { wheel_c_c.mixed_paint_dialogs.borrow_mut().remove(&id); }
                            );
                            wheel_c.mixed_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        } else {
                            PaintDisplayDialog::<A, C>::mixed_create(&paint, None, None, vec![]).show();
                        }
                    },
                    ChosenItem::TargetColour(ref colour) => {
                        let dialog = TargetColourDisplayDialog::<A>::create(&colour, None);
                        dialog.set_transient_for_from(&wheel_c.pwo());
                        dialog.show();
                    },
                    ChosenItem::None => panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );
        let wheel_c = wheel.clone();
        wheel.add_paint_item.connect_activate(
            move |_| {
                if let ChosenItem::SeriesPaint(ref paint) = *wheel_c.chosen_item.borrow() {
                    wheel_c.inform_add_series_paint(paint);
                } else if let ChosenItem::MixedPaint(ref paint) = *wheel_c.chosen_item.borrow() {
                    wheel_c.inform_add_mixed_paint(paint);
                } else {
                    panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );
        let wheel_c = wheel.clone();
        wheel.graticule.drawing_area().connect_query_tooltip(
            move |_, x, y, _, tooltip| {
                // TODO: find out why tooltip.set_tip_area() nobbles tooltips
                //let rectangle = gtk::Rectangle{x: x, y: y, width: 10, height: -10};
                //println!("Rectangle: {:?}", rectangle);
                //tooltip.set_tip_area(&rectangle);
                match wheel_c.get_item_at(Point(x as f64, y as f64)) {
                    ChosenItem::SeriesPaint(paint) => {
                        tooltip.set_text(Some(paint.tooltip_text().as_str()));
                        true
                    },
                    ChosenItem::MixedPaint(paint) => {
                        tooltip.set_text(Some(paint.tooltip_text().as_str()));
                        true
                    },
                    ChosenItem::TargetColour(colour) => {
                        tooltip.set_text(Some(colour.tooltip_text().as_str()));
                        true
                    },
                    ChosenItem::None => false,
                }
             }
        );
        let wheel_c = wheel.clone();
        wheel.graticule.connect_draw(
            move |graticule, cairo_context| {
                cairo_context.set_line_width(2.0);
                wheel_c.series_paints.draw(graticule, cairo_context);
                wheel_c.mixed_paints.draw(graticule, cairo_context);
                wheel_c.target_colours.draw(graticule, cairo_context);
            }
        );
        wheel
    }
}

impl<A, C> MixerHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    pub fn add_series_paint(&self, paint: &SeriesPaint<C>) {
        self.series_paints.add_coloured_item(paint);
    }

    pub fn remove_series_paint(&self, paint: &SeriesPaint<C>) {
        self.series_paints.remove_coloured_item(paint);
    }

    pub fn add_mixed_paint(&self, paint: &MixedPaint<C>) {
        self.mixed_paints.add_coloured_item(paint);
    }

    pub fn remove_mixed_paint(&self, paint: &MixedPaint<C>) {
        self.mixed_paints.remove_coloured_item(paint);
    }

    pub fn add_target_colour(&self, target_colour: &TargetColour) {
        self.target_colours.add_coloured_item(target_colour);
    }

    pub fn remove_target_colour(&self, target_colour: &TargetColour) {
        self.target_colours.remove_coloured_item(target_colour);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
        for dialog in self.series_paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        };
        for dialog in self.mixed_paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        };
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_item_at(&self, raw_point: Point) -> ChosenItem<C> {
        let point = self.graticule.reverse_transform(raw_point);
        let mut min_range = std::f64::MAX;
        let mut chosen_item = ChosenItem::None;
        if let Some((paint, range)) = self.series_paints.get_coloured_item_at(point) {
            if range < min_range {
                min_range = range;
                chosen_item = ChosenItem::SeriesPaint(paint);
            }
        };
        if let Some((paint, range)) = self.mixed_paints.get_coloured_item_at(point) {
            if range < min_range {
                min_range = range;
                chosen_item = ChosenItem::MixedPaint(paint);
            }
        };
        if let Some((colour, range)) = self.target_colours.get_coloured_item_at(point) {
            if range < min_range {
                chosen_item = ChosenItem::TargetColour(colour);
            }
        };
        chosen_item
    }

    pub fn connect_add_series_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.add_series_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_add_series_paint(&self, paint: &SeriesPaint<C>) {
        for callback in self.add_series_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn connect_add_mixed_paint<F: 'static + Fn(&MixedPaint<C>)>(&self, callback: F) {
        self.add_mixed_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_add_mixed_paint(&self, paint: &MixedPaint<C>) {
        for callback in self.add_mixed_paint_callbacks.borrow().iter() {
            callback(&paint);
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
