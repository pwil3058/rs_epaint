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
use gtk;
use gtk::prelude::*;

use cairox::*;
use paint::*;
use paint::shape::*;
use pwo::*;
use rgb_math::angle::*;
use rgb_math::hue::*;

pub struct Geometry {
    raw_centre: Point,
    centre: Point,
    offset: Point,
    radius: f64,
    scale: f64,
    scaled_size: f64,
    zoom: f64,
}

impl Geometry {
    pub fn new(drawing_area: &gtk::DrawingArea) -> Geometry {
        let mut geometry = Geometry{
            raw_centre: Point(0.0, 0.0),
            centre: Point(0.0, 0.0),
            offset: Point(0.0, 0.0),
            radius: 0.0,
            scale: 0.0,
            scaled_size: 0.0,
            zoom: 1.0,
        };
        geometry.update_drawing_area(drawing_area);
        geometry
    }

    pub fn transform(&self, x: f64, y: f64) -> Point {
        self.centre + Point(x, y) * self.radius
    }

    pub fn scaled_size(&self) -> f64 {
        self.scaled_size
    }

    fn update_drawing_area(&mut self, drawing_area: &gtk::DrawingArea) {
        let dw = drawing_area.get_allocated_width() as f64;
        let dh = drawing_area.get_allocated_height() as f64;

        self.raw_centre = Point(dw, dh) / 2.0;
        self.centre = self.raw_centre + self.offset;
        self.scale = dw.min(dh) / 220.0;
        self.scaled_size = self.scale * 6.0;
        self.radius = 100.0 * self.zoom * self.scale;
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
        self.radius = 100.0 * self.zoom * self.scale;
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

pub struct PaintHueAttrWheelCore<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    drawing_area: gtk::DrawingArea,
    paints: PaintShapeList<C>,
    attr: ScalarAttribute,
    geometry: Rc<RefCell<Geometry>>,
    last_xy: Cell<Point>,
    motion_enabled: Cell<bool>,
}

pub type PaintHueAttrWheel<C> = Rc<PaintHueAttrWheelCore<C>>;

impl<C> PackableWidgetInterface for PaintHueAttrWheel<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    type PackableWidgetType = gtk::DrawingArea;

    fn pwo(&self) -> gtk::DrawingArea {
        self.drawing_area.clone()
    }
}

pub trait PaintHueAttrWheelInterface<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<C>;
}

impl<C> PaintHueAttrWheelInterface<C> for PaintHueAttrWheel<C>
    where   C: Hash + Clone + PartialEq + Copy + 'static
{
    fn create(attr: ScalarAttribute) -> PaintHueAttrWheel<C> {
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_size_request(300, 300);
        let events = gdk::SCROLL_MASK | gdk::BUTTON_PRESS_MASK |
            gdk::BUTTON_MOTION_MASK | gdk::LEAVE_NOTIFY_MASK |
            gdk::BUTTON_RELEASE_MASK;
        drawing_area.add_events(events.bits() as i32);
        let paints = PaintShapeList::<C>::new(attr);
        let geometry = Rc::new(RefCell::new(Geometry::new(&drawing_area)));
        let motion_enabled = Cell::new(false);
        let last_xy: Cell<Point> = Cell::new(Point(0.0, 0.0));
        let wheel = Rc::new(
            PaintHueAttrWheelCore::<C> {
                drawing_area: drawing_area,
                paints: paints,
                attr: attr,
                geometry: geometry,
                motion_enabled: motion_enabled,
                last_xy: last_xy
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
                        let (x, y) = event.get_position();
                        wheel_c.last_xy.set(Point(x, y));
                        wheel_c.motion_enabled.set(true);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
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
        wheel
    }
}

impl<C> PaintHueAttrWheelCore<C>
    where   C: Hash + Clone + PartialEq + Copy
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
            let eol = geometry.transform(angle.cos(), angle.sin());
            cairo_context.draw_line(geometry.centre, eol);
            cairo_context.stroke();
        };
        cairo_context.set_line_width(2.0);
        self.paints.draw(&geometry, cairo_context);
    }

    pub fn add_paint(&self, paint: &Paint<C>) {
        self.paints.add_paint(paint);
    }

    pub fn get_attr(&self) -> ScalarAttribute {
        self.attr
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
