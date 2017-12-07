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
use std::path::Path;
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
use series_paint::*;
use series_paint::manager::*;
use super::collection::*;
use super::components::*;
use super::hue_wheel::*;
use super::target::*;

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
    fn create(series_paint_data_path: &Path) -> Self::PaintMixerType;
}

pub struct PaintMixerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    vbox: gtk::Box,
    cads: Rc<A>,
    colour_match_area: ColourMatchArea,
    hue_attr_wheels: Vec<MixerHueAttrWheel<A, C>>,
    paint_components: PaintComponentsBox<C>,
    mixed_paints_view: MixedPaintCollectionView<A, C>,
    notes: gtk::Entry,
    next_name_label: gtk::Label,
    mixed_paint_notes: gtk::Entry,
    // Buttons
    new_mixture_btn: gtk::Button,
    print_report_btn: gtk::Button,
    accept_mixture_btn: gtk::Button,
    reset_parts_btn: gtk::Button,
    remove_unused_btn: gtk::Button,
    simplify_parts_btn: gtk::Button,
    cancel_btn: gtk::Button,
    // Managers
    series_paint_manager: SeriesPaintManager<A, C>,
}

impl<A, C> PaintMixerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static
{
    pub fn add_series_paint(&self, paint: &SeriesPaint<C>) {
        self.paint_components.add_series_paint(paint);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.add_series_paint(paint);
        }
    }

    fn remove_series_paint(&self, paint: &SeriesPaint<C>) {
        for wheel in self.hue_attr_wheels.iter() {
            wheel.remove_series_paint(paint);
        }
    }

    fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.cads.set_target_colour(o_colour);
        self.colour_match_area.set_target_colour(o_colour);
        self.series_paint_manager.set_target_colour(o_colour);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.set_target_colour(o_colour);
        }
    }

    fn start_new_mixture(&self, o_notes: Option<&str>, o_target_colour: Option<&Colour>) {
        if let Some(notes) = o_notes {
            self.mixed_paint_notes.set_text(notes);
        } else {
            self.mixed_paint_notes.set_text("");
        }
        self.set_target_colour(o_target_colour);
        self.paint_components.reset_all_parts_to_zero();
        //if self.standards_manager:
        //    self.standards_manager.set_target_setable(False);
        let name_text = format!("#{:03}:", self.mixed_paints_view.next_mixture_id());
        self.next_name_label.set_text(name_text.as_str());
        // TODO: set button states
    }

    fn accept_new_mixture(&self) {
        let notes = if let Some(text) = self.mixed_paint_notes.get_text() {
            text
        } else {
            "".to_string()
        };
        let matched_colour = self.colour_match_area.get_target_colour();
        let components = self.paint_components.get_paint_components();
        if let Ok(mixed_paint) = self.mixed_paints_view.add_paint(&notes, &components, matched_colour) {
            for wheel in self.hue_attr_wheels.iter() {
                wheel.add_mixed_paint(&mixed_paint);
            }
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
        // TODO: set button states
    }

    fn cancel_current_mixture(&self) {
        self.mixed_paint_notes.set_text("");
        self.set_target_colour(None);
        //if self.standards_manager:
        //    self.standards_manager.set_target_setable(False);
        self.next_name_label.set_text("#00?:");
        self.paint_components.reset_all_parts_to_zero();
        // TODO: set button states
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

    fn create(series_paint_data_path: &Path) -> PaintMixer<A, C> {
        let mut view_attr_wheels:Vec<MixerHueAttrWheel<A, C>> = Vec::new();
        for attr in A::scalar_attributes().iter() {
            view_attr_wheels.push(MixerHueAttrWheel::<A, C>::create(*attr));
        }
        let mixed_paints = MixedPaintCollection::<C>::create();
        let paint_mixer = Rc::new(
            PaintMixerCore::<A, C> {
                vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
                cads: A::create(),
                hue_attr_wheels: view_attr_wheels,
                colour_match_area: ColourMatchArea::create(),
                paint_components: PaintComponentsBox::<C>::create_with(4, true),
                mixed_paints_view: MixedPaintCollectionView::<A, C>::create(&mixed_paints),
                notes: gtk::Entry::new(),
                next_name_label: gtk::Label::new(Some("#???:")),
                mixed_paint_notes: gtk::Entry::new(),
                // Buttons
                print_report_btn: gtk::Button::new_from_icon_name("gtk-print", gtk::IconSize::LargeToolbar.into()),
                new_mixture_btn: gtk::Button::new_with_label("New"),
                accept_mixture_btn: gtk::Button::new_with_label("Accept"),
                cancel_btn: gtk::Button::new_with_label("Cancel"),
                reset_parts_btn: gtk::Button::new_with_label("Reset"),
                remove_unused_btn: gtk::Button::new_with_label("Remove Unused Paints"),
                simplify_parts_btn: gtk::Button::new_with_label("Simplify Parts"),
                // Managers
                series_paint_manager: SeriesPaintManager::<A, C>::create(series_paint_data_path),
            }
        );

        // TODO: Consider redoing this when Toolbar bug fixed.
        //let toolbar = gtk::Toolbar::new();
        //let paint_mixer.print_report_btn = gtk::ToolButton::new_from_stock("gtk-print");
        //paint_mixer.print_report_btn.set_tooltip_text("Print a report of the mixtures and paints used");
        //toolbar.insert(&paint_mixer.print_report_btn.clone(), 1);
        //toolbar.insert(&paint_mixer.series_paint_manager.tool_button(), 2);
        //toolbar.show_all();
        //paint_mixer.vbox.pack_start(&toolbar, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        paint_mixer.print_report_btn.set_tooltip_text("Print a report of the mixtures and paints used");
        hbox.pack_start(&paint_mixer.print_report_btn.clone(), false, true, 2);
        hbox.pack_start(&paint_mixer.series_paint_manager.button(), false, true, 2);
        paint_mixer.vbox.pack_start(&hbox, false, false, 2);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&gtk::Label::new(Some("Notes:")), false, false, 0);
        hbox.pack_start(&paint_mixer.notes.clone(), true, true, 0);
        paint_mixer.vbox.pack_start(&hbox, false, false, 0);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&paint_mixer.next_name_label.clone(), false, false, 0);
        hbox.pack_start(&paint_mixer.mixed_paint_notes.clone(), true, true, 0);
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&paint_mixer.cads.pwo(), false, false, 0);
        vbox.pack_start(&paint_mixer.colour_match_area.pwo(), true, true, 0);

        let notebook = gtk::Notebook::new();
        for wheel in paint_mixer.hue_attr_wheels.iter() {
            let label_text = format!("Hue/{} Wheel", wheel.attr().to_string());
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

        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        paint_mixer.vbox.pack_start(&button_box.clone(), false, false, 0);
        button_box.pack_start(&paint_mixer.new_mixture_btn.clone(), true, true, 0);
        button_box.pack_start(&paint_mixer.accept_mixture_btn.clone(), true, true, 0);
        button_box.pack_start(&paint_mixer.cancel_btn.clone(), true, true, 0);
        button_box.pack_start(&paint_mixer.simplify_parts_btn.clone(), true, true, 0);
        button_box.pack_start(&paint_mixer.reset_parts_btn.clone(), true, true, 0);
        button_box.pack_start(&paint_mixer.remove_unused_btn.clone(), true, true, 0);

        paint_mixer.vbox.pack_start(&paint_mixer.mixed_paints_view.pwo(), true, true, 0);

        paint_mixer.new_mixture_btn.set_tooltip_text("Start mixing a new colour.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.new_mixture_btn.connect_clicked(
            move |_| {
                if let Some((ref notes, ref colour)) = NewTargetColourDialog::<A>::create(None).get_new_target() {
                    paint_mixer_c.start_new_mixture(Some(&notes), Some(&colour))
                }
            }
        );

        paint_mixer.accept_mixture_btn.set_tooltip_text("Accept the current mixture and add it to the list of mixed colours.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.accept_mixture_btn.connect_clicked(
            move |_| paint_mixer_c.accept_new_mixture()
        );

        paint_mixer.cancel_btn.set_tooltip_text("Cancel the current mixture.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.cancel_btn.connect_clicked(
            move |_| paint_mixer_c.cancel_current_mixture()
        );

        paint_mixer.simplify_parts_btn.set_tooltip_text("Divide all paints' parts by their greatest common denominator.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.simplify_parts_btn.connect_clicked(
            move |_| paint_mixer_c.paint_components.simplify_parts()
        );

        paint_mixer.reset_parts_btn.set_tooltip_text("Reset parts of all paints in mixing part to zero.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.reset_parts_btn.connect_clicked(
            move |_| paint_mixer_c.paint_components.reset_all_parts_to_zero()
        );

        paint_mixer.remove_unused_btn.set_tooltip_text("Remove paints with zero parts from the mixing area.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.remove_unused_btn.connect_clicked(
            move |_| paint_mixer_c.paint_components.remove_unused_spin_buttons()
        );

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.paint_components.connect_colour_changed(
            move |colour| {
                paint_mixer_c.colour_match_area.set_mixed_colour(colour);
                paint_mixer_c.cads.set_colour(colour);
            }
        );

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.paint_components.connect_paint_removed(
            move |paint| {
                if let Paint::Series(ref series_paint) = *paint {
                    paint_mixer_c.remove_series_paint(series_paint);
                }
            }
        );

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.series_paint_manager.connect_add_paint(
            move |paint| paint_mixer_c.add_series_paint(paint)
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
