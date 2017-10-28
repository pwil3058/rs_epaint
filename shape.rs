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

use cairo;

use cairox::*;
use paint::*;
use paint::hue_wheel::*;

pub trait ShapeInterface {
    fn encloses(&self, xy: Point) -> bool;
    fn distance_to(&self, xy: Point) -> f64;
    fn draw(&self, canvas: &Geometry, cairo_context: &cairo::Context);
}

const SHAPE_SIDE: f64 = 0.06;
const SHAPE_RADIUS: f64 = SHAPE_SIDE / 2.0;

pub struct PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    paint: Paint<C>,
    xy: Point,
}

impl<C> PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    pub fn create(paint: &Paint<C>, attr: ScalarAttribute) -> PaintShape<C> {
        let radius = paint.colour().scalar_attribute(attr);
        let angle = paint.colour().hue().angle().radians();
        if angle.is_nan() {
            PaintShape::<C>{
                paint: paint.clone(),
                xy: Point(0.0, radius),
            }
        } else {
            PaintShape::<C>{
                paint: paint.clone(),
                xy: Point(radius * angle.cos(), radius * angle.sin()),
            }
        }
    }

    pub fn paint(&self) -> Paint<C> {
        self.paint.clone()
    }
}

impl<C> ShapeInterface for PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn encloses(&self, xy: Point) -> bool {
        let delta_xy = self.xy - xy;
        match self.paint {
            Paint::Series(_) => delta_xy.x().abs() < SHAPE_RADIUS && delta_xy.y().abs() < SHAPE_RADIUS,
            Paint::Mixed(_) => delta_xy.hypot() < SHAPE_RADIUS
        }
    }

    fn distance_to(&self, xy: Point) -> f64 {
        (self.xy - xy).hypot()
    }

    fn draw(&self, canvas: &Geometry, cairo_context: &cairo::Context) {
        let fill_rgb = self.paint.colour().rgb();
        let outline_rgb = fill_rgb.best_foreground_rgb();
        let point = canvas.transform(self.xy);
        match self.paint {
            Paint::Series(_) => {
                let side = canvas.scaled(SHAPE_SIDE);
                cairo_context.set_source_colour_rgb(&fill_rgb);
                cairo_context.draw_square(point, side, true);
                cairo_context.set_source_colour_rgb(&outline_rgb);
                cairo_context.draw_square(point, side, false);
            },
            Paint::Mixed(_) => {
                let radius = canvas.scaled(SHAPE_RADIUS);
                cairo_context.set_source_colour_rgb(&fill_rgb);
                cairo_context.draw_circle(point, radius, true);
                cairo_context.set_source_colour_rgb(&outline_rgb);
                cairo_context.draw_circle(point, radius, false);
            }
        }
    }
}

pub struct PaintShapeList<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    attr: ScalarAttribute,
    shapes: RefCell<Vec<PaintShape<C>>>,
}

impl<C> PaintShapeList<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    pub fn new(attr: ScalarAttribute) -> PaintShapeList<C> {
        PaintShapeList::<C> {
            attr: attr,
            shapes: RefCell::new(Vec::new())
        }
    }

    pub fn len(&self) -> usize {
        self.shapes.borrow().len()
    }

    fn find_paint(&self, paint: &Paint<C>) -> Result<usize, usize> {
        self.shapes.borrow().binary_search_by_key(
            paint,
            |shape| shape.paint()
        )
    }

    pub fn contains_paint(&self, paint: &Paint<C>) -> bool {
        self.find_paint(paint).is_ok()
    }

    pub fn add_paint(&self, paint: &Paint<C>) {
        match self.find_paint(paint) {
            Ok(_) => panic!("File: {:?} Line: {:?} already includes: {:?}", file!(), line!(), paint.name()),
            Err(index) => {
                let shape = PaintShape::create(&paint, self.attr);
                self.shapes.borrow_mut().insert(index, shape);
            }
        }
    }

    pub fn remove_paint(&self, paint: &Paint<C>) {
        match self.find_paint(paint) {
            Ok(index) => {
                self.shapes.borrow_mut().remove(index);
            },
            Err(_) => panic!("File: {:?} Line: {:?} not found: {:?}", file!(), line!(), paint.name())
        }
    }

    pub fn replace_paint(&self, old_paint: &Paint<C>, new_paint: &Paint<C>) {
        self.remove_paint(old_paint);
        self.add_paint(new_paint);
    }

    pub fn draw(&self, canvas: &Geometry, cairo_context: &cairo::Context) {
        for shape in self.shapes.borrow().iter() {
            shape.draw(canvas, cairo_context);
        }
    }

    pub fn get_paint_at(&self, xy: Point) -> Option<Paint<C>> {
        let mut candidates: Vec<usize> = Vec::new();
        for (index, shape) in self.shapes.borrow().iter().enumerate() {
            if shape.encloses(xy) {
                candidates.push(index);
            }
        }
        if candidates.len() == 0 {
            None
        } else if candidates.len() == 0 {
            Some(self.shapes.borrow()[candidates[0]].paint())
        } else {
            let shapes = self.shapes.borrow();
            let mut range = shapes[candidates[0]].distance_to(xy);
            let mut index = candidates[0];
            for i in candidates[1..].iter() {
                let r = shapes[*i].distance_to(xy);
                if r < range { range = r;  index = *i; }
            }
            Some(self.shapes.borrow()[index].paint())
        }
    }
}

pub struct CurrentTargetShape {
    colour: Colour,
    xy: Point
}

impl CurrentTargetShape {
    pub fn create(colour: &Colour, attr: ScalarAttribute) -> CurrentTargetShape {
        let radius = colour.scalar_attribute(attr);
        let angle = colour.hue().angle().radians();
        if angle.is_nan() {
            CurrentTargetShape {
                colour: colour.clone(),
                xy: Point(0.0, radius),
            }
        } else {
            CurrentTargetShape {
                colour: colour.clone(),
                xy: Point(radius * angle.cos(), radius * angle.sin()),
            }
        }
    }

    pub fn colour(&self) -> Colour {
        self.colour.clone()
    }
}

impl ShapeInterface for CurrentTargetShape {
    fn encloses(&self, xy: Point) -> bool {
        let delta_xy = self.xy - xy;
        delta_xy.hypot() < SHAPE_RADIUS
    }

    fn distance_to(&self, xy: Point) -> f64 {
        (self.xy - xy).hypot()
    }

    fn draw(&self, canvas: &Geometry, cairo_context: &cairo::Context) {
        let fill_rgb = self.colour.rgb();
        let outline_rgb = fill_rgb.best_foreground_rgb();
        let point = canvas.transform(self.xy);
        let radius = canvas.scaled(SHAPE_RADIUS);
        cairo_context.set_source_colour_rgb(&fill_rgb);
        cairo_context.draw_circle(point, radius, true);
        cairo_context.set_source_colour_rgb(&outline_rgb);
        cairo_context.draw_circle(point, radius, false);

        let half_len = canvas.scaled(SHAPE_SIDE);
        let rel_end = Point(half_len, 0.0);
        cairo_context.draw_line(point + rel_end, point - rel_end);
        let rel_end = Point(0.0, half_len);
        cairo_context.draw_line(point + rel_end, point - rel_end);
    }
}


#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
