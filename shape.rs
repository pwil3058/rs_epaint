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
use std::rc::Rc;

use cairo;
use gtk;

use cairox::*;
use paint::*;

pub struct CanvasContext {
    centre_x: f64,
    centre_y: f64,
    zoom: f64,
    scaled_size: f64,
}

impl CanvasContext {
    pub fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        (self.centre_x + x * self.zoom, self.centre_y + y * self.zoom)
    }
}

pub trait ShapeInterface {
    fn encloses(&self, x:f64, y: f64, scaled_size: f64) -> bool;
    fn distance_to(&self, x: f64, y: f64) -> f64;
    fn draw(&self, canvas: &CanvasContext, cairo_context: &cairo::Context);
}

pub struct PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    paint: Paint<C>,
    x : f64,
    y : f64,
}

impl<C> PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    pub fn create(paint: Paint<C>, attr: ScalarAttribute) -> PaintShape<C> {
        let radius = paint.colour().scalar_attribute(attr);
        let angle = paint.colour().hue().angle().radians();
        if angle.is_nan() {
            let x = 0.0;
            let y = radius;
            PaintShape::<C>{paint, x, y}
        } else {
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            PaintShape::<C>{paint, x, y}
        }
    }

    pub fn paint(&self) -> Paint<C> {
        self.paint.clone()
    }
}

impl<C> ShapeInterface for PaintShape<C>
    where   C: Hash + Clone + PartialEq + Copy
{
    fn encloses(&self, x:f64, y: f64, scaled_size: f64) -> bool {
        let delta_x = self.x - x;
        let delta_y = self.y - y;
        match self.paint {
            Paint::Series(_) => delta_x.abs() < scaled_size && delta_y.abs() < scaled_size,
            Paint::Mixed(_) => delta_x.hypot(delta_y) < scaled_size
        }
    }

    fn distance_to(&self, x: f64, y: f64) -> f64 {
        (self.x -x).hypot(self.y - y)
    }

    fn draw(&self, canvas: &CanvasContext, cairo_context: &cairo::Context) {
        let fill_rgb = self.paint.colour().rgb();
        let outline_rgb = fill_rgb.best_foreground_rgb();
        let (x, y) = canvas.transform(self.x, self.y);
        match self.paint {
            Paint::Series(_) => {
                cairo_context.set_source_colour_rgb(&fill_rgb);
                cairo_context.draw_square((x, y), canvas.scaled_size, true);
                cairo_context.set_source_colour_rgb(&outline_rgb);
                cairo_context.draw_square((x, y), canvas.scaled_size, false);
            },
            Paint::Mixed(_) => {
                cairo_context.set_source_colour_rgb(&fill_rgb);
                cairo_context.draw_circle((x, y), canvas.scaled_size, true);
                cairo_context.set_source_colour_rgb(&outline_rgb);
                cairo_context.draw_circle((x, y), canvas.scaled_size, false);
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
    fn find_paint(&self, paint: &Paint<C>) -> Result<usize, usize> {
        self.shapes.borrow().binary_search_by_key(
            paint,
            |shape| shape.paint()
        )
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
