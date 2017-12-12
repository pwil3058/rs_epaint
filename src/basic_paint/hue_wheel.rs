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

use basic_paint::*;
use basic_paint::collection::{CollectionIdInterface, CollnPaintInterface};
use graticule::*;
use shape::*;

// SHAPE
pub struct CollnPaintShape<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>,
{
    paint: P,
    xy: Point,
    phantom: PhantomData<C>,
    phantom_1: PhantomData<CID>,
}

impl<C, P, CID> ColourShapeInterface for CollnPaintShape<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>,
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

impl<C, P, CID> ColouredItemShapeInterface<P> for CollnPaintShape<C, P, CID>
    where   C: CharacteristicsInterface,
            CID: CollectionIdInterface,
            P: CollnPaintInterface<C, CID>,
{
    fn new(paint: &P, attr: ScalarAttribute) -> CollnPaintShape<C, P, CID> {
        let radius = paint.scalar_attribute(attr);
        let angle = paint.hue().angle();
        CollnPaintShape::<C, P, CID>{
            paint: paint.clone(),
            xy: Point::from((angle, radius)),
            phantom: PhantomData,
            phantom_1: PhantomData,
        }
    }

    fn coloured_item(&self) -> P {
        self.paint.clone()
    }
}

pub type CollnPaintShapeList<C, P, CID> = ColouredItemSpapeList<P, CollnPaintShape<C, P, CID>>;


// WHEEL
pub struct CollnPaintHueAttrWheelCore<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface + 'static,
            P: CollnPaintInterface<C, CID>,
{
    paints: CollnPaintShapeList<C, P, CID>,
    chosen_paint: RefCell<Option<P>>,
    graticule: Graticule,
    phantom: PhantomData<CID>,
}

pub type CollnPaintHueAttrWheel<C, P, CID> = Rc<CollnPaintHueAttrWheelCore<C, P, CID>>;

pub trait CollnPaintHueAttrWheelInterface<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface + 'static,
            P: CollnPaintInterface<C, CID>,
{
    fn pwo(&self) -> gtk::DrawingArea;
    fn create(attr: ScalarAttribute) -> CollnPaintHueAttrWheel<C, P, CID>;
}

impl<C, P, CID> CollnPaintHueAttrWheelInterface<C, P, CID> for CollnPaintHueAttrWheel<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface + 'static,
            P: CollnPaintInterface<C, CID> + 'static,
{
    fn pwo(&self) -> gtk::DrawingArea {
        self.graticule.drawing_area()
    }

    fn create(attr: ScalarAttribute) -> CollnPaintHueAttrWheel<C, P, CID> {
        let wheel = Rc::new(
            CollnPaintHueAttrWheelCore::<C, P, CID> {
                paints: CollnPaintShapeList::<C, P, CID>::new(attr),
                graticule: Graticule::create(attr),
                chosen_paint: RefCell::new(None),
                phantom: PhantomData,
            }
        );
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

impl<C, P, CID> CollnPaintHueAttrWheelCore<C, P, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollectionIdInterface + 'static,
            P: CollnPaintInterface<C, CID>,
{
    pub fn add_paint(&self, paint: &P) {
        self.paints.add_coloured_item(paint);
    }

    pub fn remove_paint(&self, paint: &P) {
        self.paints.remove_coloured_item(paint);
    }

    pub fn replace_paint(&self, old_paint: &P, paint: &P) {
        self.paints.replace_coloured_item(old_paint, paint);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_paint_at(&self, raw_point: Point) -> Option<P> {
        let point = self.graticule.reverse_transform(raw_point);
        let opr = self.paints.get_coloured_item_at(point);
        if let Some((paint, _)) = opr {
            Some(paint)
        } else {
            None
        }
    }

    pub fn set_chosen_paint_from(&self, raw_point: Point) -> Option<P> {
        if let Some(paint) = self.get_paint_at(raw_point) {
            *self.chosen_paint.borrow_mut() = Some(paint.clone());
            Some(paint)
        } else {
            *self.chosen_paint.borrow_mut() = None;
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
