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
use std::fmt::Debug;
use std::marker::PhantomData;

use cairo;

use pw_gix::cairox::*;
use pw_gix::colour::*;
use pw_gix::rgb_math::rgb::*;

use basic_paint::*;
use mixed_paint::*;
use mixed_paint::target::*;
use series_paint::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ShapeType {
    Circle,
    Diamond,
    Square,
    BackSight,
}

pub trait GeometryInterface {
    fn transform(&self, point: Point) -> Point;
    fn reverse_transform(&self, point: Point) -> Point;
    fn scaled(&self, value: f64) -> f64 ;
}

const SHAPE_SIDE: f64 = 0.06;
const SHAPE_RADIUS: f64 = SHAPE_SIDE / 2.0;

pub trait ColourShapeInterface {
    fn xy(&self) -> Point;
    fn fill_rgb(&self) -> RGB;
    fn shape_type(&self) -> ShapeType;

    fn encloses(&self, xy: Point) -> bool {
        match self.shape_type() {
            ShapeType::Square => {
                let delta_xy = self.xy() - xy;
                delta_xy.x().abs() < SHAPE_RADIUS && delta_xy.y().abs() < SHAPE_RADIUS
            },
            ShapeType::Diamond => {
                let delta_xy = (self.xy() - xy).rotate_45_deg();
                delta_xy.x().abs() < SHAPE_RADIUS && delta_xy.y().abs() < SHAPE_RADIUS
            },
            _ => {
                (self.xy() - xy).hypot() < SHAPE_RADIUS
            },
        }
    }

    fn distance_to(&self, xy: Point) -> f64 {
        (self.xy() - xy).hypot()
    }

    fn draw<G: GeometryInterface>(&self, canvas: &G, cairo_context: &cairo::Context) {
        let fill_rgb = self.fill_rgb();
        let outline_rgb = fill_rgb.best_foreground_rgb();
        let point = canvas.transform(self.xy());
        let side = canvas.scaled(SHAPE_SIDE);
        match self.shape_type() {
           ShapeType::Square => {
                cairo_context.set_source_colour_rgb(fill_rgb);
                cairo_context.draw_square(point, side, true);
                cairo_context.set_source_colour_rgb(outline_rgb);
                cairo_context.draw_square(point, side, false);
            },
            ShapeType::Diamond => {
                cairo_context.set_source_colour_rgb(fill_rgb);
                cairo_context.draw_diamond(point, side, true);
                cairo_context.set_source_colour_rgb(outline_rgb);
                cairo_context.draw_diamond(point, side, false);
            },
            ShapeType::Circle => {
                let radius = canvas.scaled(SHAPE_RADIUS);
                cairo_context.set_source_colour_rgb(fill_rgb);
                cairo_context.draw_circle(point, radius, true);
                cairo_context.set_source_colour_rgb(outline_rgb);
                cairo_context.draw_circle(point, radius, false);
            },
            ShapeType::BackSight => {
                let radius = canvas.scaled(SHAPE_RADIUS);
                cairo_context.set_source_colour_rgb(fill_rgb);
                cairo_context.draw_circle(point, radius, true);
                cairo_context.set_source_colour_rgb(outline_rgb);
                cairo_context.draw_circle(point, radius, false);

                let half_len = canvas.scaled(SHAPE_SIDE);
                let rel_end = Point(half_len, 0.0);
                cairo_context.draw_line(point + rel_end, point - rel_end);
                let rel_end = Point(0.0, half_len);
                cairo_context.draw_line(point + rel_end, point - rel_end);
            },
        }
    }
}

pub trait ColouredItemShapeInterface<CI>: ColourShapeInterface
    where   CI: ColouredItemInterface + Ord
{
    fn new(paint: &CI, attr: ScalarAttribute) -> Self;
    fn coloured_item(&self) -> CI;
}

pub struct ColouredItemSpapeList<CI, PS>
    where   CI: ColouredItemInterface + Ord,
            PS: ColouredItemShapeInterface<CI>,
{
    attr: ScalarAttribute,
    shapes: RefCell<Vec<PS>>,
    changed_callbacks: RefCell<Vec<Box<Fn()>>>,
    pc: PhantomData<CI>,
}

impl<CI, PS> ColouredItemSpapeList<CI, PS>
        where   CI: ColouredItemInterface + Ord + Debug,
                PS: ColouredItemShapeInterface<CI>,
{
    pub fn new(attr: ScalarAttribute) -> ColouredItemSpapeList<CI, PS> {
        ColouredItemSpapeList::<CI, PS> {
            attr: attr,
            shapes: RefCell::new(Vec::new()),
            changed_callbacks: RefCell::new(Vec::new()),
            pc: PhantomData
        }
    }

    pub fn clear(&self) {
        self.shapes.borrow_mut().clear()
    }

    pub fn len(&self) -> usize {
        self.shapes.borrow().len()
    }

    fn find_coloured_item(&self, coloured_item: &CI) -> Result<usize, usize> {
        self.shapes.borrow().binary_search_by_key(
            coloured_item,
            |shape| shape.coloured_item()
        )
    }

    pub fn contains_coloured_item(&self, coloured_item: &CI) -> bool {
        self.find_coloured_item(coloured_item).is_ok()
    }

    pub fn add_coloured_item(&self, coloured_item: &CI) {
        if let Err(index) = self.find_coloured_item(coloured_item) {
            let shape = PS::new(coloured_item, self.attr);
            self.shapes.borrow_mut().insert(index, shape);
            self.inform_changed();
        } else {
            // we already contain this paint so quietly ignore
        }
    }

    pub fn remove_coloured_item(&self, coloured_item: &CI) {
        match self.find_coloured_item(coloured_item) {
            Ok(index) => {
                self.shapes.borrow_mut().remove(index);
                self.inform_changed();
            },
            Err(_) => panic!("File: {:?} Line: {:?} not found: {:?}", file!(), line!(), coloured_item)
        }
    }

    pub fn replace_coloured_item(&self, old_coloured_item: &CI, new_coloured_item: &CI) {
        self.remove_coloured_item(old_coloured_item);
        self.add_coloured_item(new_coloured_item);
    }

    pub fn draw<G: GeometryInterface>(&self, canvas: &G, cairo_context: &cairo::Context) {
        for shape in self.shapes.borrow().iter() {
            shape.draw(canvas, cairo_context);
        }
    }

    pub fn get_coloured_item_at(&self, xy: Point) -> Option<(CI, f64)> {
        let mut candidates: Vec<usize> = Vec::new();
        for (index, shape) in self.shapes.borrow().iter().enumerate() {
            if shape.encloses(xy) {
                candidates.push(index);
            }
        }
        if candidates.len() == 0 {
            None
        } else {
            let shapes = self.shapes.borrow();
            let mut range = shapes[candidates[0]].distance_to(xy);
            let mut index = candidates[0];
            for i in candidates[1..].iter() {
                let r = shapes[*i].distance_to(xy);
                if r < range { range = r;  index = *i; }
            }
            Some((self.shapes.borrow()[index].coloured_item(), range))
        }
    }

    pub fn connect_changed<F: 'static + Fn()>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_changed(&self) {
        for callback in self.changed_callbacks.borrow().iter() {
            callback();
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
