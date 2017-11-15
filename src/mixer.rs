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

use pw_gix::cairox::*;

use pw_gix::pwo::*;
use pw_gix::colour::*;
use pw_gix::gtkx::paned::*;
use pw_gix::rgb_math::rgb::*;

use paint::*;
use components::*;
use hue_wheel::*;
use series_paint::*;

trait ColourMatchAreaInterface {
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
            cairo_context.set_source_colour_rgb(BLACK);
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

implement_pwo!(ColourMatchAreaCore, drawing_area, gtk::DrawingArea);

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
        };
        self.drawing_area.queue_draw();
    }

    fn set_target_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            *self.target_colour.borrow_mut() = Some(colour.clone())
        } else {
            *self.target_colour.borrow_mut() = None
        };
        self.drawing_area.queue_draw();
    }
}

pub trait PaintMixerInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    type PaintMixerType;

    fn pwo(&self) -> gtk::Box;
    fn create() -> Self::PaintMixerType;
}

pub struct PaintMixerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    vbox: gtk::Box,
    cads: Rc<A>,
    colour_match_area: ColourMatchArea,
    hue_attr_wheels: Vec<PaintHueAttrWheel<A, C>>,
    paint_components: PaintComponentsBox<C>,
}

impl<A, C> PaintMixerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    pub fn add_series_paint(&self, paint: &SeriesPaint<C>) {
        self.paint_components.add_series_paint(paint);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.add_paint(&Paint::Series(paint.clone()));
        }
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        self.cads.set_target_colour(ocolour);
        self.colour_match_area.set_target_colour(ocolour);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.set_target_colour(ocolour);
        }
    }
}

pub type PaintMixer<A, C> = Rc<PaintMixerCore<A, C>>;

impl<A, C> PaintMixerInterface<A, C> for PaintMixer<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    type PaintMixerType = PaintMixer<A, C>;

    fn pwo(&self) -> gtk::Box {
        self.vbox.clone()
    }

    fn create() -> PaintMixer<A, C> {
        let mut view_attr_wheels:Vec<PaintHueAttrWheel<A, C>> = Vec::new();
        for attr in A::scalar_attributes().iter() {
            view_attr_wheels.push(PaintHueAttrWheel::<A, C>::create(*attr));
        }
        let paint_mixer = Rc::new(
            PaintMixerCore::<A, C> {
                vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
                cads: A::create(),
                hue_attr_wheels: view_attr_wheels,
                colour_match_area: ColourMatchArea::create(),
                paint_components: PaintComponentsBox::<C>::create_with(6, true),
            }
        );
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        vbox.pack_start(&paint_mixer.cads.pwo(), true, true, 0);
        vbox.pack_start(&paint_mixer.colour_match_area.pwo(), true, true, 0);

        let notebook = gtk::Notebook::new();
        for wheel in paint_mixer.hue_attr_wheels.iter() {
            let label_text = format!("Hue/{} Wheel", wheel.get_attr().to_string());
            let label = gtk::Label::new(Some(label_text.as_str()));
            notebook.append_page(&wheel.pwo(), Some(&label));
        }

        let hpaned = gtk::Paned::new(gtk::Orientation::Horizontal);
        hpaned.pack1(&notebook, true, true);
        hpaned.pack2(&vbox, true, true);
        hpaned.set_position_from_recollections("paint_mixer_horizontal", 200);
        paint_mixer.vbox.pack_start(&hpaned, true, true, 0);

        let frame = gtk::Frame::new(Some("Paints"));
        frame.add(&paint_mixer.paint_components.pwo());
        paint_mixer.vbox.pack_start(&frame, true, true, 0);
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
