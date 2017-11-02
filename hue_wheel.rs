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
use std::rc::Rc;

use cairo;
use gdk;
use gdk::prelude::*;
use gtk;
use gtk::prelude::*;

use cairox::*;
use colour::attributes::*;
use paint::*;
use paint::series::*;
use paint::shape::*;
use paint::target::*;
use pwo::*;
use rgb_math::angle::*;
use rgb_math::hue::*;

pub struct Geometry {
    raw_centre: Point,
    centre: Point,
    offset: Point,
    radius: f64,
    scaled_one: f64,
    zoom: f64,
}

impl Geometry {
    pub fn new(drawing_area: &gtk::DrawingArea) -> Geometry {
        let mut geometry = Geometry{
            raw_centre: Point(0.0, 0.0),
            centre: Point(0.0, 0.0),
            offset: Point(0.0, 0.0),
            radius: 0.0,
            scaled_one: 0.0,
            zoom: 1.0,
        };
        geometry.update_drawing_area(drawing_area);
        geometry
    }

    pub fn transform(&self, point: Point) -> Point {
        self.centre + point * self.radius
    }

    pub fn reverse_transform(&self, point: Point) -> Point {
        (point - self.centre) / self.radius
    }

    pub fn scaled(&self, value: f64) -> f64 {
        value * self.scaled_one
    }

    fn update_drawing_area(&mut self, drawing_area: &gtk::DrawingArea) {
        let dw = drawing_area.get_allocated_width() as f64;
        let dh = drawing_area.get_allocated_height() as f64;

        self.raw_centre = Point(dw, dh) / 2.0;
        self.centre = self.raw_centre + self.offset;
        self.scaled_one = dw.min(dh) / 2.2;
        self.radius = self.zoom * self.scaled_one;
    }

    fn shift_offset(&mut self, delta_xy: Point) {
        self.offset += delta_xy;
        self.centre = self.raw_centre + self.offset;
    }

    fn set_zoom(&mut self, zoom: f64) {
        let new_zoom = zoom.max(1.0).min(10.0);
        let ratio = new_zoom / self.zoom;
        self.offset *= ratio;
        self.centre = self.raw_centre + self.offset;
        self.zoom = new_zoom;
        self.radius = self.zoom * self.scaled_one;
    }

    fn decr_zoom(&mut self) {
        let new_zoom = self.zoom - 0.025;
        self.set_zoom(new_zoom)
    }

    fn incr_zoom(&mut self) {
        let new_zoom = self.zoom + 0.025;
        self.set_zoom(new_zoom)
    }
}

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
pub struct PaintHueAttrWheelCore<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    drawing_area: gtk::DrawingArea,
    menu: gtk::Menu,
    paint_info_item: gtk::MenuItem,
    add_paint_item: gtk::MenuItem,
    paints: PaintShapeList<C>,
    target_colours: TargetColourShapeList,
    current_target: RefCell<Option<CurrentTargetShape>>,
    chosen_item: RefCell<ChosenItem<C>>,
    attr: ScalarAttribute,
    geometry: Rc<RefCell<Geometry>>,
    last_xy: Cell<Point>,
    motion_enabled: Cell<bool>,
    add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>>,
    series_paint_dialogs: RefCell<HashMap<u32, SeriesPaintDisplayDialog<C, CADS>>>,
}

pub type PaintHueAttrWheel<C, CADS> = Rc<PaintHueAttrWheelCore<C, CADS>>;

impl<C, CADS> PackableWidgetInterface for PaintHueAttrWheel<C, CADS>
    where   C: CharacteristicsInterface,
            CADS: ColourAttributeDisplayStackInterface
{
    type PackableWidgetType = gtk::DrawingArea;

    fn pwo(&self) -> gtk::DrawingArea {
        self.drawing_area.clone()
    }
}

pub trait PaintHueAttrWheelInterface<C, CADS>
    where   C: CharacteristicsInterface + 'static,
            CADS: ColourAttributeDisplayStackInterface + 'static
{
    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<C, CADS>;
}

impl<C, CADS> PaintHueAttrWheelInterface<C, CADS> for PaintHueAttrWheel<C, CADS>
    where   C: CharacteristicsInterface + 'static,
            CADS: ColourAttributeDisplayStackInterface + 'static
{
    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<C, CADS> {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_size_request(300, 300);
        drawing_area.set_has_tooltip(true);
        let events = gdk::SCROLL_MASK | gdk::BUTTON_PRESS_MASK |
            gdk::BUTTON_MOTION_MASK | gdk::LEAVE_NOTIFY_MASK |
            gdk::BUTTON_RELEASE_MASK;
        drawing_area.add_events(events.bits() as i32);
        let menu = gtk::Menu::new();
        let paint_info_item = gtk::MenuItem::new_with_label("Information");
        menu.append(&paint_info_item.clone());
        let add_paint_item = gtk::MenuItem::new_with_label("Add to Mixer");
        add_paint_item.set_visible(false);
        menu.append(&add_paint_item.clone());
        menu.show_all();
        let paints = PaintShapeList::<C>::new(attr);
        let target_colours = TargetColourShapeList::new(attr);
        let current_target: RefCell<Option<CurrentTargetShape>> = RefCell::new(None);
        let geometry = Rc::new(RefCell::new(Geometry::new(&drawing_area)));
        let motion_enabled = Cell::new(false);
        let last_xy: Cell<Point> = Cell::new(Point(0.0, 0.0));
        let add_paint_callbacks: RefCell<Vec<Box<Fn(&SeriesPaint<C>)>>> = RefCell::new(Vec::new());
        let series_paint_dialogs: RefCell<HashMap<u32, SeriesPaintDisplayDialog<C, CADS>>> = RefCell::new(HashMap::new());
        let wheel = Rc::new(
            PaintHueAttrWheelCore::<C, CADS> {
                drawing_area: drawing_area,
                menu: menu,
                paint_info_item: paint_info_item.clone(),
                add_paint_item: add_paint_item.clone(),
                paints: paints,
                target_colours: target_colours,
                current_target: current_target,
                attr: attr,
                geometry: geometry,
                motion_enabled: motion_enabled,
                last_xy: last_xy,
                chosen_item: RefCell::new(ChosenItem::None),
                add_paint_callbacks: add_paint_callbacks,
                series_paint_dialogs: series_paint_dialogs,
            }
        );
        let wheel_c = wheel.clone();
        wheel.drawing_area.connect_draw(
            move |_, cc| {wheel_c.draw(cc); Inhibit(false)}
        );
        let geometry_c = wheel.geometry.clone();
        wheel.drawing_area.connect_configure_event(
            move |da, _| {geometry_c.borrow_mut().update_drawing_area(da); false}
        );
        let geometry_c = wheel.geometry.clone();
        wheel.drawing_area.connect_scroll_event(
            move |da, scroll_event| {
                if let Some(device) = scroll_event.get_device() {
                    if device.get_source() == gdk::InputSource::Mouse {
                        match scroll_event.get_direction() {
                            gdk::ScrollDirection::Up => {
                                geometry_c.borrow_mut().decr_zoom();
                                da.queue_draw();
                                return Inhibit(true);
                            },
                            gdk::ScrollDirection::Down => {
                                geometry_c.borrow_mut().incr_zoom();
                                da.queue_draw();
                                return Inhibit(true);
                            },
                            _ => return Inhibit(false)
                        }
                    }
                }
                Inhibit(false)
            }
        );
        let wheel_c = wheel.clone();
        wheel.drawing_area.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 1 {
                        let point = Point::from(event.get_position());
                        wheel_c.last_xy.set(point);
                        wheel_c.motion_enabled.set(true);
                        return Inhibit(true)
                    } else if event.get_button() == 3 {
                        let chosen_item = wheel_c.get_item_at(Point::from(event.get_position()));
                        wheel_c.paint_info_item.set_sensitive(!chosen_item.is_none());
                        wheel_c.add_paint_item.set_sensitive(chosen_item.is_series_paint());
                        let have_listeners = wheel_c.add_paint_callbacks.borrow().len() > 0;
                        wheel_c.add_paint_item.set_visible(have_listeners);
                        *wheel_c.chosen_item.borrow_mut() = chosen_item;
                        wheel_c.menu.popup_at_pointer(None);
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
                            Paint::Series(ref paint) => {
                                let target = if let Some(ref current_target) = *wheel_c.current_target.borrow() {
                                    Some(current_target.colour())
                                } else {
                                    None
                                };
                                let have_listeners = wheel_c.add_paint_callbacks.borrow().len() > 0;
                                let buttons = if have_listeners {
                                    let wheel_c_c = wheel_c.clone();
                                    let paint_c = paint.clone();
                                    let spec = SeriesPaintDisplayButtonSpec {
                                        label: "Add".to_string(),
                                        tooltip_text: "Add this paint to the paint mixing area.".to_string(),
                                        callback:  Box::new(move || wheel_c_c.inform_add_paint(&paint_c))
                                    };
                                    vec![spec]
                                } else {
                                    vec![]
                                };
                                let dialog = SeriesPaintDisplayDialog::<C, CADS>::create(
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
                            Paint::Mixed(ref paint) => {
                                println!("Show information for: {:?}", paint);
                            }
                        }
                    },
                    ChosenItem::TargetColour(ref colour) => {
                        println!("Show information for: {:?}", colour);
                        TargetColourDisplayDialog::<CADS>::create(&colour, None).show();
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
        wheel.drawing_area.connect_motion_notify_event(
            move |da, event| {
                if wheel_c.motion_enabled.get() {
                    let (x, y) = event.get_position();
                    let this_xy = Point(x, y);
                    let delta_xy = this_xy - wheel_c.last_xy.get();
                    wheel_c.last_xy.set(this_xy);
                    wheel_c.geometry.borrow_mut().shift_offset(delta_xy);
                    da.queue_draw();
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
             }
        );
        let wheel_c = wheel.clone();
        wheel.drawing_area.connect_button_release_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonRelease {
                    if event.get_button() == 1 {
                        wheel_c.motion_enabled.set(false);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );
        let wheel_c = wheel.clone();
        wheel.drawing_area.connect_leave_notify_event(
            move |_, _| {
                wheel_c.motion_enabled.set(false);
                Inhibit(false)
             }
        );
        let wheel_c = wheel.clone();
        wheel.drawing_area.connect_query_tooltip(
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
        wheel
    }
}

impl<C, CADS> PaintHueAttrWheelCore<C, CADS>
    where   C: CharacteristicsInterface + 'static,
            CADS: ColourAttributeDisplayStackInterface + 'static
{
    fn draw(&self, cairo_context: &cairo::Context) {
        let geometry = self.geometry.borrow();

        cairo_context.set_source_colour_rgb(&(WHITE * 0.5));
        cairo_context.paint();

        cairo_context.set_source_colour_rgb(&(WHITE * 0.75));
        let n_rings: u8 = 10;
        for i in 0..n_rings {
            let radius = geometry.radius * (i as f64 + 1.0) / n_rings as f64;
            cairo_context.draw_circle(geometry.centre, radius, false);
        };

        cairo_context.set_line_width(4.0);
        for i in 0..6 {
            let angle = DEG_60 * i;
            let hue = HueAngle::from(angle);
            cairo_context.set_source_colour_rgb(&hue.max_chroma_rgb());
            let eol = geometry.transform(Point::from((angle, 1.0)));
            cairo_context.draw_line(geometry.centre, eol);
            cairo_context.stroke();
        };
        cairo_context.set_line_width(2.0);
        self.paints.draw(&geometry, cairo_context);
        self.target_colours.draw(&geometry, cairo_context);
        if let Some(ref current_target) = *self.current_target.borrow() {
            current_target.draw(&geometry, cairo_context);
        }
    }

    pub fn add_paint(&self, paint: &Paint<C>) {
        self.paints.add_paint(paint);
    }

    pub fn add_target_colour(&self, target_colour: &TargetColour) {
        self.target_colours.add_target_colour(target_colour);
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        match ocolour {
            Some(colour) => {
                let current_target = CurrentTargetShape::create(colour, self.attr);
                for dialog in self.series_paint_dialogs.borrow().values() {
                    dialog.set_current_target(Some(colour.clone()));
                };
                *self.current_target.borrow_mut() = Some(current_target)
            },
            None => {
                for dialog in self.series_paint_dialogs.borrow().values() {
                    dialog.set_current_target(None);
                };
                *self.current_target.borrow_mut() = None
            },
        }
    }

    pub fn get_attr(&self) -> ScalarAttribute {
        self.attr
    }

    pub fn get_item_at(&self, raw_point: Point) -> ChosenItem<C> {
        let geometry = self.geometry.borrow();
        let point = geometry.reverse_transform(raw_point);
        let opr = self.paints.get_paint_at(point);
        let ocr = self.target_colours.get_target_colour_at(point);
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
