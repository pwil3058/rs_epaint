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

use pw_gix::cairox::*;
use pw_gix::colour::*;
use pw_gix::rgb_math::angle::*;
use pw_gix::rgb_math::hue::*;
use pw_gix::rgb_math::rgb::*;

use display::*;
use paint::*;
use series_paint::*;
use shape::*;
use target::*;

pub struct GraticuleCore {
    drawing_area: gtk::DrawingArea,
    attr: ScalarAttribute,
    raw_centre: Cell<Point>,
    centre: Cell<Point>,
    offset: Cell<Point>,
    radius: Cell<f64>,
    scaled_one: Cell<f64>,
    zoom: Cell<f64>,
    current_target: RefCell<Option<CurrentTargetShape>>,
    last_xy: Cell<Point>,
    motion_enabled: Cell<bool>,
    draw_callbacks: RefCell<Vec<Box<Fn(&GraticuleCore, &cairo::Context)>>>,
}

impl GeometryInterface for GraticuleCore {
    fn transform(&self, point: Point) -> Point {
        self.centre.get() + point * self.radius.get()
    }

    fn reverse_transform(&self, point: Point) -> Point {
        (point - self.centre.get()) / self.radius.get()
    }

    fn scaled(&self, value: f64) -> f64 {
        value * self.scaled_one.get()
    }
}

pub trait GraticuleInterface {
    fn create(attr: ScalarAttribute) -> Rc<GraticuleCore>;
}

impl GraticuleCore {
    pub fn attr(&self) -> ScalarAttribute {
        self.attr
    }

    pub fn drawing_area(&self) -> gtk::DrawingArea {
        self.drawing_area.clone()
    }

    fn update_drawing_area(&self) {
        let dw = self.drawing_area.get_allocated_width() as f64;
        let dh = self.drawing_area.get_allocated_height() as f64;

        self.raw_centre.set(Point(dw, dh) / 2.0);
        self.centre.set(self.raw_centre.get() + self.offset.get());
        self.scaled_one.set(dw.min(dh) / 2.2);
        self.radius.set(self.zoom.get() * self.scaled_one.get());
    }

    fn shift_offset(&self, delta_xy: Point) {
        self.offset.set(self.offset.get() + delta_xy);
        self.centre.set(self.raw_centre.get() + self.offset.get());
    }

    fn set_zoom(&self, zoom: f64) {
        let new_zoom = zoom.max(1.0).min(10.0);
        let ratio = new_zoom / self.zoom.get();
        self.offset.set(self.offset.get() * ratio);
        self.centre.set(self.raw_centre.get() + self.offset.get());
        self.zoom.set(new_zoom);
        self.radius.set(self.zoom.get() * self.scaled_one.get());
    }

    fn decr_zoom(&self) {
        let new_zoom = self.zoom.get() - 0.025;
        self.set_zoom(new_zoom)
    }

    fn incr_zoom(&self) {
        let new_zoom = self.zoom.get() + 0.025;
        self.set_zoom(new_zoom)
    }

    fn draw(&self, cairo_context: &cairo::Context) {
        cairo_context.set_source_colour_rgb((WHITE * 0.5));
        cairo_context.paint();

        cairo_context.set_source_colour_rgb((WHITE * 0.75));
        let n_rings: u8 = 10;
        for i in 0..n_rings {
            let radius = self.radius.get() * (i as f64 + 1.0) / n_rings as f64;
            cairo_context.draw_circle(self.centre.get(), radius, false);
        };

        cairo_context.set_line_width(4.0);
        for i in 0..6 {
            let angle = DEG_60 * i;
            let hue = HueAngle::from(angle);
            cairo_context.set_source_colour_rgb(hue.max_chroma_rgb());
            let eol = self.transform(Point::from((angle, 1.0)));
            cairo_context.draw_line(self.centre.get(), eol);
            cairo_context.stroke();
        };
        cairo_context.set_line_width(2.0);
        for callback in self.draw_callbacks.borrow().iter(){
            callback(self, cairo_context);
        }
        if let Some(ref current_target) = *self.current_target.borrow() {
            current_target.draw(self, cairo_context);
        }
    }

    pub fn connect_draw<F: 'static + Fn(&GraticuleCore, &cairo::Context)>(&self, callback: F) {
        self.draw_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn set_current_target_colour(&self, o_colour: Option<&Colour>) {
        if let Some(colour) = o_colour {
            *self.current_target.borrow_mut() = Some(CurrentTargetShape::create(&colour, self.attr));
        } else {
            *self.current_target.borrow_mut() = None;
        }
    }

    pub fn current_target_colour(&self) -> Option<Colour> {
        if let Some(ref shape) = *self.current_target.borrow() {
            Some(shape.colour().clone())
        } else {
            None
        }
    }
}

impl GraticuleInterface for Rc<GraticuleCore> {
    fn create(attr: ScalarAttribute) -> Rc<GraticuleCore> {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_size_request(300, 300);
        drawing_area.set_has_tooltip(true);
        let events = gdk::SCROLL_MASK | gdk::BUTTON_PRESS_MASK |
            gdk::BUTTON_MOTION_MASK | gdk::LEAVE_NOTIFY_MASK |
            gdk::BUTTON_RELEASE_MASK;
        drawing_area.add_events(events.bits() as i32);
        let graticule = Rc::new(
            GraticuleCore{
                drawing_area: drawing_area,
                attr: attr,
                raw_centre: Cell::new(Point(0.0, 0.0)),
                centre: Cell::new(Point(0.0, 0.0)),
                offset: Cell::new(Point(0.0, 0.0)),
                radius: Cell::new(0.0),
                scaled_one: Cell::new(0.0),
                zoom: Cell::new(1.0),
                current_target: RefCell::new(None),
                motion_enabled: Cell::new(false),
                last_xy: Cell::new(Point(0.0, 0.0)),
                draw_callbacks: RefCell::new(Vec::new()),
            }
        );
        graticule.update_drawing_area();
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_draw(
            move |_, cc| { graticule_c.draw(cc); Inhibit(false) }
        );
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_configure_event(
            move |_, _| {graticule_c.update_drawing_area(); false}
        );
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_scroll_event(
            move |da, scroll_event| {
                if let Some(device) = scroll_event.get_device() {
                    if device.get_source() == gdk::InputSource::Mouse {
                        match scroll_event.get_direction() {
                            gdk::ScrollDirection::Up => {
                                graticule_c.decr_zoom();
                                da.queue_draw();
                                return Inhibit(true);
                            },
                            gdk::ScrollDirection::Down => {
                                graticule_c.incr_zoom();
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

        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 1 {
                        let point = Point::from(event.get_position());
                        graticule_c.last_xy.set(point);
                        graticule_c.motion_enabled.set(true);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_motion_notify_event(
            move |da, event| {
                if graticule_c.motion_enabled.get() {
                    let (x, y) = event.get_position();
                    let this_xy = Point(x, y);
                    let delta_xy = this_xy - graticule_c.last_xy.get();
                    graticule_c.last_xy.set(this_xy);
                    graticule_c.shift_offset(delta_xy);
                    da.queue_draw();
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
             }
        );
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_button_release_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonRelease {
                    if event.get_button() == 1 {
                        graticule_c.motion_enabled.set(false);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );
        let graticule_c = graticule.clone();
        graticule.drawing_area.connect_leave_notify_event(
            move |_, _| {
                graticule_c.motion_enabled.set(false);
                Inhibit(false)
             }
        );
        graticule
    }
}

type Graticule = Rc<GraticuleCore>;

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
