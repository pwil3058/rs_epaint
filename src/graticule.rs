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

use cairo;
use gdk;
use gdk::prelude::*;
use glib::signal::SignalHandlerId;
use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;
use pw_gix::colour::*;
use pw_gix::rgb_math::angle::*;
use pw_gix::rgb_math::hue::*;
use pw_gix::rgb_math::rgb::*;

use shape::*;

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

    pub fn queue_draw(&self) {
        self.drawing_area.queue_draw()
    }

    pub fn connect_draw<F: 'static + Fn(&GraticuleCore, &cairo::Context)>(&self, callback: F) {
        self.draw_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn set_current_target_colour(&self, o_colour: Option<&Colour>) {
        if let Some(colour) = o_colour {
            *self.current_target.borrow_mut() = Some(CurrentTargetShape::create(&colour, self.attr));
        } else {
            *self.current_target.borrow_mut() = None;
        };
        self.queue_draw()
    }

    pub fn current_target_colour(&self) -> Option<Colour> {
        if let Some(ref shape) = *self.current_target.borrow() {
            Some(shape.colour().clone())
        } else {
            None
        }
    }

    pub fn connect_button_press_event<F: Fn(&gtk::DrawingArea, &gdk::EventButton) -> Inhibit + 'static>(&self, f: F) -> SignalHandlerId {
        self.drawing_area.connect_button_press_event(f)
    }
}

impl GraticuleInterface for Rc<GraticuleCore> {
    fn create(attr: ScalarAttribute) -> Rc<GraticuleCore> {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_size_request(300, 300);
        drawing_area.set_has_tooltip(true);
        let events = gdk::EventMask::SCROLL_MASK | gdk::EventMask::BUTTON_PRESS_MASK |
            gdk::EventMask::BUTTON_MOTION_MASK | gdk::EventMask::LEAVE_NOTIFY_MASK |
            gdk::EventMask::BUTTON_RELEASE_MASK;
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

pub type Graticule = Rc<GraticuleCore>;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
