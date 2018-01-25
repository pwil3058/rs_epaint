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
use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;

pub use pw_gix::wrapper::*;
use pw_gix::colour::*;
use pw_gix::gtkx::menu::*;
use pw_gix::rgb_math::rgb::*;

use super::*;

struct Sample {
    pix_buf: Pixbuf,
    position: Point,
}

pub struct ColourMatchAreaCore {
    drawing_area: gtk::DrawingArea,
    mixed_colour: RefCell<Option<Colour>>,
    target_colour: RefCell<Option<Colour>>,
    popup_menu: WrappedMenu,
    samples: RefCell<Vec<Sample>>,
    popup_menu_position: Cell<Point>,
    mixing_mode: MixingMode,
}

impl ColourMatchAreaCore {
    pub fn mixing_mode(&self) -> MixingMode {
        self.mixing_mode
    }

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
        for sample in self.samples.borrow().iter() {
            cairo_context.set_source_pixbuf_at(&sample.pix_buf, sample.position, false);
            cairo_context.paint();
        }
    }

    pub fn get_target_colour(&self) -> Option<Colour> {
        if let Some(ref colour) = *self.target_colour.borrow() {
            Some(colour.clone())
        } else {
            None
        }
    }

    pub fn has_target_colour(&self) -> bool {
        self.target_colour.borrow().is_some()
    }

    pub fn set_mixed_colour(&self, colour: Option<&Colour>) {
        if let Some(colour) = colour {
            *self.mixed_colour.borrow_mut() = Some(colour.clone())
        } else {
            *self.mixed_colour.borrow_mut() = None
        };
        self.drawing_area.queue_draw();
    }

    pub fn set_target_colour(&self, colour: Option<&Colour>) {
        assert!(colour.is_none() || self.mixing_mode == MixingMode::MatchTarget);
        if let Some(colour) = colour {
            *self.target_colour.borrow_mut() = Some(colour.clone())
        } else {
            *self.target_colour.borrow_mut() = None
        };
        self.drawing_area.queue_draw();
    }

    pub fn remove_samples(&self) {
        self.samples.borrow_mut().clear();
        self.drawing_area.queue_draw();
    }

}

pub type ColourMatchArea = Rc<ColourMatchAreaCore>;

impl_widget_wrapper!(ColourMatchAreaCore, drawing_area, gtk::DrawingArea);

pub trait ColourMatchAreaInterface {
    type ColourMatchAreaType;

    fn create(mixing_mode: MixingMode) -> Self::ColourMatchAreaType;
}

impl ColourMatchAreaInterface for ColourMatchArea {
    type ColourMatchAreaType = ColourMatchArea;

    fn create(mixing_mode: MixingMode) -> ColourMatchArea {
        let colour_match_area = Rc::new(
            ColourMatchAreaCore {
                drawing_area: gtk::DrawingArea::new(),
                mixed_colour: RefCell::new(None),
                target_colour: RefCell::new(None),
                popup_menu: WrappedMenu::new(&vec![]),
                samples: RefCell::new(Vec::new()),
                popup_menu_position: Cell::new(Point(0.0, 0.0)),
                mixing_mode: mixing_mode
            }
        );

        if mixing_mode == MixingMode::MatchSamples {
            let colour_match_area_c = colour_match_area.clone();
            colour_match_area.popup_menu.append_item(
                "paste",
                "Paste Sample",
                "Paste image sample from the clipboard at this position",
            ).connect_activate(
                move |_| {
                    let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                    if let Some(pixbuf) = cbd.wait_for_image() {
                        let sample = Sample{pix_buf: pixbuf, position: colour_match_area_c.popup_menu_position.get()};
                        colour_match_area_c.samples.borrow_mut().push(sample);
                        colour_match_area_c.drawing_area.queue_draw();
                    } else {
                        colour_match_area_c.inform_user("No image data on clipboard.", None);
                    }
                }
            );

            let colour_match_area_c = colour_match_area.clone();
            colour_match_area.popup_menu.append_item(
                "remove",
                "Remove Sample(s)",
                "Remove all image samples from the sample area",
            ).connect_activate(
                move |_| {
                    colour_match_area_c.remove_samples();
                }
            );
        };

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
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn paint_mixer_test() {
        //assert!(false)
    }
}