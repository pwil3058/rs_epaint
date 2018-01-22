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
use pw_gix::gtkx::menu::*;
use pw_gix::rgb_math::rgb::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use dialogue::*;
use graticule::*;
use series_paint::*;
use shape::*;

use super::*;
use super::display::*;
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

// SERIES PAINT SHAPE
pub struct SeriesPaintShape<C: CharacteristicsInterface> {
    paint: SeriesPaint<C>,
    xy: Point,
}

impl<C: CharacteristicsInterface> ColourShapeInterface for SeriesPaintShape<C> {
    fn xy(&self) -> Point {
        self.xy
    }

    fn fill_rgb(&self) -> RGB {
        self.paint.rgb()
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Square
    }
}

impl<C> ColouredItemShapeInterface<SeriesPaint<C>> for SeriesPaintShape<C>
    where   C: CharacteristicsInterface
{
    fn new(paint: &SeriesPaint<C>, attr: ScalarAttribute) -> SeriesPaintShape<C> {
        let radius = paint.scalar_attribute(attr);
        let angle = paint.hue().angle();
        SeriesPaintShape::<C>{
            paint: paint.clone(),
            xy: Point::from((angle, radius)),
        }
    }

    fn coloured_item(&self) -> SeriesPaint<C> {
        self.paint.clone()
    }
}

pub type SeriesPaintShapeList<C> = ColouredItemSpapeList<SeriesPaint<C>, SeriesPaintShape<C>>;

// MIXED PAINT SHAPE
pub struct MixedPaintShape<C: CharacteristicsInterface> {
    paint: MixedPaint<C>,
    xy: Point,
}

impl<C: CharacteristicsInterface> ColourShapeInterface for MixedPaintShape<C> {
    fn xy(&self) -> Point {
        self.xy
    }

    fn fill_rgb(&self) -> RGB {
        self.paint.rgb()
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Diamond
    }
}

impl<C> ColouredItemShapeInterface<MixedPaint<C>> for MixedPaintShape<C>
    where   C: CharacteristicsInterface
{
    fn new(paint: &MixedPaint<C>, attr: ScalarAttribute) -> MixedPaintShape<C> {
        let radius = paint.scalar_attribute(attr);
        let angle = paint.hue().angle();
        MixedPaintShape::<C>{
            paint: paint.clone(),
            xy: Point::from((angle, radius)),
        }
    }

    fn coloured_item(&self) -> MixedPaint<C> {
        self.paint.clone()
    }
}

pub type MixedPaintShapeList<C> = ColouredItemSpapeList<MixedPaint<C>, MixedPaintShape<C>>;

// TARGET COLOUR SHAPE

pub struct TargetColourShape {
    target_colour: TargetColour,
    xy: Point,
}

impl ColourShapeInterface for TargetColourShape {
    fn xy(&self) -> Point {
        self.xy
    }

    fn fill_rgb(&self) -> RGB {
        self.target_colour.rgb()
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }
}

impl ColouredItemShapeInterface<TargetColour> for TargetColourShape {
    fn new(target_colour: &TargetColour, attr: ScalarAttribute) -> TargetColourShape {
        let radius = target_colour.colour().scalar_attribute(attr);
        let angle = target_colour.colour().hue().angle();
        TargetColourShape{
            target_colour: target_colour.clone(),
            xy: Point::from((angle, radius)),
        }
    }

    fn coloured_item(&self) -> TargetColour {
        self.target_colour.clone()
    }
}

pub type TargetColourShapeList = ColouredItemSpapeList<TargetColour, TargetColourShape>;

// WHEEL
pub struct MixerHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    popup_menu: WrappedMenu,
    series_paints: SeriesPaintShapeList<C>,
    mixed_paints: MixedPaintShapeList<C>,
    target_colours: TargetColourShapeList,
    chosen_item: RefCell<ChosenItem<C>>,
    graticule: Graticule,
    add_series_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    add_mixed_paint_callbacks: RefCell<Vec<Box<Fn(&MixedPaint<C>)>>>,
    series_paint_dialogs: RefCell<HashMap<u32, SeriesPaintDisplayDialog<A, C>>>,
    mixed_paint_dialogs: RefCell<HashMap<u32, MixedPaintDisplayDialog<A, C>>>,
}

impl<A, C> WidgetWrapper for MixerHueAttrWheelCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    type PWT = gtk::DrawingArea;

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
        let wheel = Rc::new(
            MixerHueAttrWheelCore::<A, C> {
                popup_menu: WrappedMenu::new(&vec![]),
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
        wheel.popup_menu.append_item(
            "info",
            "Paint Information",
            "Display this paint's information",
        ).connect_activate(
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
                            let dialog = SeriesPaintDisplayDialog::<A, C>::create(&paint, target, &wheel_c, vec![spec]);
                            let wheel_c_c = wheel_c.clone();
                            dialog.connect_destroyed(
                                move |id| { wheel_c_c.series_paint_dialogs.borrow_mut().remove(&id); }
                            );
                            wheel_c.series_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        } else {
                            SeriesPaintDisplayDialog::<A, C>::create(&paint, target, &wheel_c, vec![]).show();
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
                            let dialog = MixedPaintDisplayDialog::<A, C>::create(&paint, target, &wheel_c, vec![spec]);
                            let wheel_c_c = wheel_c.clone();
                            dialog.connect_destroy(
                                move |id| { wheel_c_c.mixed_paint_dialogs.borrow_mut().remove(&id); }
                            );
                            wheel_c.mixed_paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        } else {
                            MixedPaintDisplayDialog::<A, C>::create(&paint, None, &wheel_c, vec![]).show();
                        }
                    },
                    ChosenItem::TargetColour(ref colour) => {
                        let dialog = TargetColourDisplayDialog::<A>::create(&colour, &wheel_c);
                        dialog.show();
                    },
                    ChosenItem::None => panic!("File: {:?} Line: {:?} SHOULDN'T GET HERE", file!(), line!())
                }
            }
        );

        let wheel_c = wheel.clone();
        wheel.popup_menu.append_item(
            "add",
            "Add to Mixer",
            "Add this paint to the mixer palette",
        ).connect_activate(
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
        wheel.graticule.drawing_area().connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let chosen_item = wheel_c.get_item_at(Point::from(event.get_position()));
                        wheel_c.popup_menu.set_sensitivities(!chosen_item.is_none(), &["info"]);
                        let have_series_listeners = wheel_c.add_series_paint_callbacks.borrow().len() > 0;
                        let have_mixed_listeners = wheel_c.add_mixed_paint_callbacks.borrow().len() > 0;
                        wheel_c.popup_menu.set_visibilities(have_series_listeners || have_mixed_listeners, &["add"]);
                        if chosen_item.is_series_paint() {
                            wheel_c.popup_menu.set_sensitivities(have_series_listeners, &["add"]);
                        } else if chosen_item.is_mixed_paint() {
                            wheel_c.popup_menu.set_sensitivities(have_mixed_listeners, &["add"]);
                        } else {
                            wheel_c.popup_menu.set_sensitivities(false, &["add"]);
                        };
                        *wheel_c.chosen_item.borrow_mut() = chosen_item;
                        wheel_c.popup_menu.popup_at_event(event);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
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
        if let Some(ref target_colour) = paint.target_colour() {
            self.target_colours.add_coloured_item(target_colour);
        }
    }

    pub fn remove_mixed_paint(&self, paint: &MixedPaint<C>) {
        self.mixed_paints.remove_coloured_item(paint);
        if let Some(ref target_colour) = paint.target_colour() {
            self.target_colours.remove_coloured_item(target_colour);
        }
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
