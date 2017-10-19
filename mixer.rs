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
use gtk::prelude::*;

use cairox::*;

use pwo::*;
use colour::attributes::*;
use paint::*;
use paint::components::*;
//use rgb_math::rgb::*;

trait ColourMatchAreaInterface: PackableWidgetInterface {
    type ColourMatchAreaType;

    fn create() -> Self::ColourMatchAreaType;
    fn clear(&self);

    fn get_mixed_colour(&self) -> Option<Colour>;
    fn get_target_colour(&self) -> Option<Colour>;

    fn set_mixed_colour(&self, colour: Option<&Colour>);
    fn set_target_colour(&self, colour: Option<&Colour>);
}

struct ColourMatchAreaCore {
    drawing_area: gtk::DrawingArea,
    mixed_colour: RefCell<Option<Colour>>,
    target_colour: RefCell<Option<Colour>>,
}

impl ColourMatchAreaCore {
    fn draw(
        &self,
        drawing_area: &gtk::DrawingArea,
        cairo_context: &cairo::Context
    ) {
        if let Some(ref colour) = *self.mixed_colour.borrow() {
            cairo_context.set_source_colour(&colour);
        } else {
            cairo_context.set_source_colour_rgb(&BLACK);
        };
        cairo_context.paint();
        if let Some(ref colour) = *self.target_colour.borrow() {
            cairo_context.set_source_colour(&colour);
            let width = drawing_area.get_allocated_width() as f64;
            let height = drawing_area.get_allocated_height() as f64;
            cairo_context.rectangle(
                width / 4.0, height / 4.0, width / 2.0, height / 2.0
            );
            cairo_context.fill();
        }
    }
}

type ColourMatchArea = Rc<ColourMatchAreaCore>;

implement_pwo!(ColourMatchArea, drawing_area, gtk::DrawingArea);

impl ColourMatchAreaInterface for ColourMatchArea {
    type ColourMatchAreaType = ColourMatchArea;

    fn create() -> ColourMatchArea {
        let colour_match_area = Rc::new(
            ColourMatchAreaCore {
                drawing_area: gtk::DrawingArea::new(),
                mixed_colour: RefCell::new(None),
                target_colour: RefCell::new(None)
            }
        );
        let colour_match_area_c = colour_match_area.clone();
        colour_match_area.drawing_area.connect_draw(
            move |da, ctxt|
            {
                colour_match_area_c.draw(da, ctxt);
                Inhibit(false)
            }
        );
        colour_match_area
    }

    fn clear(&self) {
        *self.mixed_colour.borrow_mut() = None;
        *self.target_colour.borrow_mut() = None;
        self.drawing_area.queue_draw();
    }

    fn get_mixed_colour(&self) -> Option<Colour> {
        if let Some(ref colour) = *self.mixed_colour.borrow() {
            Some(colour.clone())
        } else {
            None
        }
    }

    fn get_target_colour(&self) -> Option<Colour> {
        if let Some(ref colour) = *self.target_colour.borrow() {
            Some(colour.clone())
        } else {
            None
        }
    }

    fn set_mixed_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            *self.mixed_colour.borrow_mut() = Some(colour.clone())
        } else {
            *self.mixed_colour.borrow_mut() = None
        }
    }

    fn set_target_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            *self.target_colour.borrow_mut() = Some(colour.clone())
        } else {
            *self.target_colour.borrow_mut() = None
        }
    }
}

pub trait PaintMixerInterface<CADS, C>: PackableWidgetInterface
    where   CADS: ColourAttributeDisplayStackInterface,
            C: Hash + Clone + PartialEq + Copy
{
    type PaintMixerType;

    fn create() -> Self::PaintMixerType;
}

pub struct PaintMixerCore<CADS, C>
    where   CADS: ColourAttributeDisplayStackInterface,
            C: Hash + Clone + PartialEq + Copy
{
    vbox: gtk::Box,
    cads: CADS,
    colour_match_area: ColourMatchArea,
    paint_components: PaintComponentsBox<C>,
}

pub type PaintMixer<CADS, C> = Rc<PaintMixerCore<CADS, C>>;

impl<CADS, C> PackableWidgetInterface for PaintMixer<CADS, C>
    where   CADS: ColourAttributeDisplayStackInterface,
            C: Hash + Clone + PartialEq + Copy
{
    type PackableWidgetType = gtk::Box;

    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }
}

impl<CADS, C> PaintMixerInterface<CADS, C> for PaintMixer<CADS, C>
    where   CADS: ColourAttributeDisplayStackInterface + 'static,
            C: Hash + Clone + PartialEq + Copy + 'static
{
    type PaintMixerType = PaintMixer<CADS, C>;

    fn create() -> PaintMixer<CADS, C> {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let paint_mixer = Rc::new(
            PaintMixerCore::<CADS, C> {
                vbox: vbox.clone(),
                cads: CADS::create(),
                colour_match_area: ColourMatchArea::create(),
                paint_components: PaintComponentsBox::<C>::create_with(6, true),
            }
        );
        vbox.pack_start(&paint_mixer.cads.pwo(), true, true, 0);
        vbox.pack_start(&paint_mixer.colour_match_area.pwo(), true, true, 0);
        vbox.pack_start(&paint_mixer.paint_components.pwo(), true, true, 0);
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.paint_components.connect_colour_changed(
            move |colour| {
                paint_mixer_c.colour_match_area.set_mixed_colour(colour);
                paint_mixer_c.cads.set_colour(colour);
            }
        );
        paint_mixer
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn paint_mixer_test() {
        //assert!(false)
    }
}
