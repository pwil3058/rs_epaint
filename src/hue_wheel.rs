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
use std::collections::HashMap;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;
use pw_gix::colour::*;

use display::*;
use graticule::*;
use paint::*;
use series_paint::*;
use shape::*;
use mixed_paint::target::*;

// CHOSEN_ITEM
#[derive(Debug)]
pub enum ChosenItem<C: CharacteristicsInterface> {
    Paint(Paint<C>),
    TargetColour(TargetColour),
    None
}

impl<C: CharacteristicsInterface> ChosenItem<C> {
    pub fn is_paint(&self) -> bool {
        match *self {
            ChosenItem::Paint(_) => true,
            _ => false
        }
    }

    pub fn is_series_paint(&self) -> bool {
        match *self {
            ChosenItem::Paint(ref paint) => paint.is_series(),
            _ => false
        }
    }
    pub fn is_mixed_paint(&self) -> bool {
        match *self {
            ChosenItem::Paint(ref paint) => paint.is_mixed(),
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
pub struct PaintHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    menu: gtk::Menu,
    paint_info_item: gtk::MenuItem,
    add_paint_item: gtk::MenuItem,
    paints: PaintShapeList<C>,
    target_colours: TargetColourShapeList,
    chosen_item: RefCell<ChosenItem<C>>,
    graticule: Graticule,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    series_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
}

pub type PaintHueAttrWheel<A, C> = Rc<PaintHueAttrWheelCore<A, C>>;

pub trait PaintHueAttrWheelInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn pwo(&self) -> gtk::DrawingArea;
    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<A, C>;
}

impl<A, C> PaintHueAttrWheelInterface<A, C> for PaintHueAttrWheel<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn pwo(&self) -> gtk::DrawingArea {
        self.graticule.drawing_area()
    }

    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<A, C> {
        let menu = gtk::Menu::new();
        let paint_info_item = gtk::MenuItem::new_with_label("Information");
        menu.append(&paint_info_item.clone());
        let add_paint_item = gtk::MenuItem::new_with_label("Add to Mixer");
        add_paint_item.set_visible(false);
        menu.append(&add_paint_item.clone());
        menu.show_all();
        let paints = PaintShapeList::<C>::new(attr);
        let target_colours = TargetColourShapeList::new(attr);
        let graticule = Graticule::create(attr);
        let add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>> = RefCell::new(Vec::new());
        let series_paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>> = RefCell::new(HashMap::new());
        let wheel = Rc::new(
            PaintHueAttrWheelCore::<A, C> {
                menu: menu,
                paint_info_item: paint_info_item.clone(),
                add_paint_item: add_paint_item.clone(),
                paints: paints,
                target_colours: target_colours,
                graticule: graticule,
                chosen_item: RefCell::new(ChosenItem::None),
                add_paint_callbacks: add_paint_callbacks,
                series_paint_dialogs: series_paint_dialogs,
            }
        );
        let wheel_c = wheel.clone();
        wheel.graticule.drawing_area().connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let chosen_item = wheel_c.get_item_at(Point::from(event.get_position()));
                        wheel_c.paint_info_item.set_sensitive(!chosen_item.is_none());
                        wheel_c.add_paint_item.set_sensitive(chosen_item.is_series_paint());
                        let have_listeners = wheel_c.add_paint_callbacks.borrow().len() > 0;
                        wheel_c.add_paint_item.set_visible(have_listeners);
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
        paint_info_item.clone().connect_activate(
            move |_| {
                match *wheel_c.chosen_item.borrow() {
                    ChosenItem::Paint(ref paint) => {
                        match *paint {
                            Paint::Series(ref series_paint) => {
                                let target_colour = wheel_c.graticule.current_target_colour().clone();
                                let target = if let Some(ref colour) = target_colour {
                                    Some(colour)
                                } else {
                                    None
                                };
                                let have_listeners = wheel_c.add_paint_callbacks.borrow().len() > 0;
                                let buttons = if have_listeners {
                                    let wheel_c_c = wheel_c.clone();
                                    let paint_c = series_paint.clone();
                                    let spec = PaintDisplayButtonSpec {
                                        label: "Add".to_string(),
                                        tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                                        callback:  Box::new(move || wheel_c_c.inform_add_paint(&paint_c))
                                    };
                                    vec![spec]
                                } else {
                                    vec![]
                                };
                                let dialog = PaintDisplayDialog::<A, C>::create(
                                    &paint,
                                    target,
                                    None,
                                    buttons
                                );
                                let wheel_c_c = wheel_c.clone();
                                dialog.connect_destroy(
                                    move |id| { wheel_c_c.series_paint_dialogs.borrow_mut().remove(&id); }
                                );
                                wheel_c.series_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                                dialog.show();
                            },
                            Paint::Mixed(ref mixed_paint) => {
                                println!("Show information for: {:?}", mixed_paint);
                                PaintDisplayDialog::<A, C>::create(&paint, None, None, vec![]).show();
                            }
                        }
                    },
                    ChosenItem::TargetColour(ref colour) => {
                        TargetColourDisplayDialog::<A>::create(&colour, None).show();
                    },
                    ChosenItem::None => panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );
        let wheel_c = wheel.clone();
        add_paint_item.clone().connect_activate(
            move |_| {
                if let ChosenItem::Paint(ref paint) = *wheel_c.chosen_item.borrow() {
                    if let Paint::Series(ref series_paint) = *paint {
                        wheel_c.inform_add_paint(series_paint);
                    } else {
                        panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                    }
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
                    ChosenItem::Paint(paint) => {
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
                wheel_c.paints.draw(graticule, cairo_context);
                wheel_c.target_colours.draw(graticule, cairo_context);
            }
        );
        wheel
    }
}

impl<A, C> PaintHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    pub fn add_paint(&self, paint: &Paint<C>) {
        self.paints.add_coloured_item(paint);
    }

    pub fn add_target_colour(&self, target_colour: &TargetColour) {
        self.target_colours.add_coloured_item(target_colour);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
        for dialog in self.series_paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        };
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_item_at(&self, raw_point: Point) -> ChosenItem<C> {
        let point = self.graticule.reverse_transform(raw_point);
        let opr = self.paints.get_coloured_item_at(point);
        let ocr = self.target_colours.get_coloured_item_at(point);
        if let Some((paint, p_range)) = opr {
            if let Some((colour, c_range)) = ocr {
                if c_range < p_range {
                    ChosenItem::TargetColour(colour)
                } else {
                    ChosenItem::Paint(paint)
                }
            } else {
                ChosenItem::Paint(paint)
            }
        } else if let Some((colour, _)) = ocr {
            ChosenItem::TargetColour(colour)
        } else {
            ChosenItem::None
        }
    }

    pub fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.add_paint_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_add_paint(&self, paint: &SeriesPaint<C>) {
        for callback in self.add_paint_callbacks.borrow().iter() {
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
