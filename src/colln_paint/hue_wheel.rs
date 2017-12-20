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
use std::marker::PhantomData;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;
use pw_gix::colour::*;
use pw_gix::rgb_math::rgb::*;

use basic_paint::*;
use graticule::*;
use shape::*;

use super::{CollnIdInterface, CollnPaint};

pub struct CollnPaintShape<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
    paint: CollnPaint<C, CID>,
    xy: Point,
}

impl<C, CID> ColourShapeInterface for CollnPaintShape<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
{
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

impl<C, CID> ColouredItemShapeInterface<CollnPaint<C, CID>> for CollnPaintShape<C, CID>
    where   C: CharacteristicsInterface,
            CID: CollnIdInterface,
{
    fn new(paint: &CollnPaint<C, CID>, attr: ScalarAttribute) -> CollnPaintShape<C, CID> {
        let radius = paint.scalar_attribute(attr);
        let angle = paint.hue().angle();
        CollnPaintShape::<C, CID>{
            paint: paint.clone(),
            xy: Point::from((angle, radius)),
        }
    }

    fn coloured_item(&self) -> CollnPaint<C, CID> {
        self.paint.clone()
    }
}

pub type CollnPaintShapeList<C, CID> = ColouredItemSpapeList<CollnPaint<C, CID>, CollnPaintShape<C, CID>>;


// WHEEL
pub struct CollnPaintHueAttrWheelCore<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    paints: CollnPaintShapeList<C, CID>,
    graticule: Graticule,
}

pub type CollnPaintHueAttrWheel<C, CID> = Rc<CollnPaintHueAttrWheelCore<C, CID>>;

pub trait CollnPaintHueAttrWheelInterface<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn pwo(&self) -> gtk::DrawingArea;
    fn create(attr: ScalarAttribute, paints: Rc<Vec<CollnPaint<C, CID>>>) -> CollnPaintHueAttrWheel<C, CID>;
}

impl<C, CID> CollnPaintHueAttrWheelInterface<C, CID> for CollnPaintHueAttrWheel<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn pwo(&self) -> gtk::DrawingArea {
        self.graticule.drawing_area()
    }

    fn create(attr: ScalarAttribute, paints: Rc<Vec<CollnPaint<C, CID>>>) -> CollnPaintHueAttrWheel<C, CID> {
        let wheel = Rc::new(
            CollnPaintHueAttrWheelCore::<C, CID> {
                paints: CollnPaintShapeList::<C, CID>::new(attr),
                graticule: Graticule::create(attr),
            }
        );
        for paint in paints.iter() {
            wheel.add_paint(paint)
        }

        let wheel_c = wheel.clone();
        wheel.graticule.drawing_area().connect_query_tooltip(
            move |_, x, y, _, tooltip| {
                // TODO: find out why tooltip.set_tip_area() nobbles tooltips
                //let rectangle = gtk::Rectangle{x: x, y: y, width: 10, height: -10};
                //println!("Rectangle: {:?}", rectangle);
                //tooltip.set_tip_area(&rectangle);
                match wheel_c.get_paint_at(Point(x as f64, y as f64)) {
                    Some(paint) => {
                        tooltip.set_text(Some(paint.tooltip_text().as_str()));
                        true
                    },
                    None => false,
                }
             }
        );
        let wheel_c = wheel.clone();
        wheel.graticule.connect_draw(
            move |graticule, cairo_context| {
                cairo_context.set_line_width(2.0);
                wheel_c.paints.draw(graticule, cairo_context);
            }
        );
        wheel
    }
}

impl<C, CID> CollnPaintHueAttrWheelCore<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn add_paint(&self, paint: &CollnPaint<C, CID>) {
        self.paints.add_coloured_item(paint);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_paint_at(&self, raw_point: Point) -> Option<CollnPaint<C, CID>> {
        let point = self.graticule.reverse_transform(raw_point);
        let opr = self.paints.get_coloured_item_at(point);
        if let Some((paint, _)) = opr {
            Some(paint)
        } else {
            None
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
