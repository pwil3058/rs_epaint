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

use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use num::Integer;

use chrono::prelude::*;
use xml::escape::*;

use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::paned::*;
pub use pw_gix::wrapper::*;

use basic_paint::*;
use colour_mix::*;
use icons::mixtures_print_xpm;
use series_paint::*;
use standards::*;

use super::collection::*;
use super::components::*;
use super::hue_wheel::*;
use super::match_area::*;
use super::target::*;
use super::*;

pub trait MixerConfig {
    fn mixing_mode() -> MixingMode;
}

pub trait PaintMixerInterface<A, C, MC>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    MC: MixerConfig + 'static,
{
    fn create(
        series_paint_data_path: &Path,
        paint_standards_data_path: Option<&Path>,
    ) -> PaintMixer<A, C, MC>;
}

pub type SeriesPaintComponentBox<A, C> =
    PaintComponentsBox<A, C, SeriesPaint<C>, SeriesPaintDisplayDialog<A, C>>;

pub struct PaintMixerCore<A, C, MC>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    MC: MixerConfig + 'static,
{
    vbox: gtk::Box,
    cads: Rc<A>,
    colour_match_area: ColourMatchArea,
    hue_attr_wheels: Vec<MixerHueAttrWheel<A, C>>,
    series_paint_components: SeriesPaintComponentBox<A, C>,
    mixed_paints: MixedPaintCollectionWidget<A, C>,
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
    o_paint_standards_manager: Option<PaintStandardManager<A, C>>,
    phantom: PhantomData<MC>,
}

impl<A, C, MC> PaintMixerCore<A, C, MC>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    MC: MixerConfig + 'static,
{
    pub fn set_manager_icons(&self, icon: &Pixbuf) {
        self.series_paint_manager.set_icon(icon);
        if let Some(ref saint_standards_manager) = self.o_paint_standards_manager {
            saint_standards_manager.set_icon(icon);
        }
    }

    fn has_notes(&self) -> bool {
        if let Some(text) = self.mixed_paint_notes.get_text() {
            text.len() > 0
        } else {
            false
        }
    }

    pub fn add_series_paint(&self, paint: &SeriesPaint<C>) {
        self.series_paint_components.add_paint(paint);
        for wheel in self.hue_attr_wheels.iter() {
            wheel.add_series_paint(paint);
        }
    }

    fn handle_series_paint_removal_request(&self, paint: &SeriesPaint<C>) {
        //TODO: implement different policies for what "unused" means
        let users = self.mixed_paints.mixed_paints_using_series_paint(paint);
        if users.is_empty() {
            self.series_paint_components.remove_paint(paint);
            for wheel in self.hue_attr_wheels.iter() {
                wheel.remove_series_paint(paint);
            }
        } else {
            let expln = format!(
                "\"{}\" is being used in one or more mixtures.",
                paint.name()
            );
            self.warn_user("Removal aborted!", Some(&expln))
        }
    }

    fn remove_unused_paints_from_mixing_area(&self) {
        //TODO: implement different policies for what "unused" means
        let series_paints_in_use = self.mixed_paints.series_paints_used();
        for paint in self
            .series_paint_components
            .remove_unused_spin_buttons(&series_paints_in_use)
            .iter()
        {
            for wheel in self.hue_attr_wheels.iter() {
                wheel.remove_series_paint(paint);
            }
        }
    }

    fn remove_mixed_paint(&self, paint: &MixedPaint<C>) {
        let message = format!("Confirm remove {}: {}", paint.name(), paint.notes());
        if self.ask_confirm_action(&message, None) {
            if let Err(err) = self.mixed_paints.remove_paint(paint) {
                let message = format!("Error: {}: {}", paint.name(), paint.notes());
                self.report_error(&message, &err);
            } else {
                for wheel in self.hue_attr_wheels.iter() {
                    wheel.remove_mixed_paint(paint);
                }
            }
        }
    }

    fn set_button_sensitivities(&self) {
        let has_colour = self.series_paint_components.has_contributions()
            || self.mixed_paints.components().has_contributions();
        self.simplify_parts_btn.set_sensitive(has_colour);
        self.reset_parts_btn.set_sensitive(has_colour);
        if MC::mixing_mode() == MixingMode::MatchSamples {
            self.accept_mixture_btn
                .set_sensitive(has_colour && self.has_notes());
        } else if self.colour_match_area.has_target_colour() {
            self.series_paint_components.set_sensitive(true);
            self.mixed_paints.components().set_sensitive(true);
            self.new_mixture_btn.set_sensitive(false);
            self.cancel_btn.set_sensitive(true);
            self.accept_mixture_btn
                .set_sensitive(has_colour && self.has_notes());
            if let Some(ref paint_standards_manager) = self.o_paint_standards_manager {
                paint_standards_manager.set_initiate_select_ok(false)
            };
        } else {
            self.series_paint_components.set_sensitive(false);
            self.mixed_paints.components().set_sensitive(false);
            self.new_mixture_btn.set_sensitive(true);
            self.accept_mixture_btn.set_sensitive(false);
            self.cancel_btn.set_sensitive(false);
            if let Some(ref paint_standards_manager) = self.o_paint_standards_manager {
                paint_standards_manager.set_initiate_select_ok(true)
            };
        };
    }

    fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.cads.set_target_colour(o_colour);
        self.colour_match_area.set_target_colour(o_colour);
        self.series_paint_manager.set_target_colour(o_colour);
        self.series_paint_components.set_current_target(o_colour);
        self.mixed_paints.set_target_colour(o_colour);
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
        self.series_paint_components.reset_all_parts_to_zero();
        self.mixed_paints.components().reset_all_parts_to_zero();
        let name_text = format!("#{:03}:", self.mixed_paints.next_mixture_id());
        self.next_name_label.set_text(name_text.as_str());
        self.set_button_sensitivities();
    }

    fn accept_new_mixture(&self) {
        let notes: String = if let Some(text) = self.mixed_paint_notes.get_text() {
            String::from(text)
        } else {
            "".to_string()
        };
        let o_matched_colour = self.colour_match_area.get_target_colour();
        let sp_components = self.series_paint_components.get_paint_components();
        let mp_components = self.mixed_paints.components().get_paint_components();
        if let Ok(mixed_paint) =
            self.mixed_paints
                .add_paint(&notes, sp_components, mp_components, o_matched_colour)
        {
            for wheel in self.hue_attr_wheels.iter() {
                wheel.add_mixed_paint(&mixed_paint);
            }
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
        self.cancel_current_mixture();
    }

    fn update_mixed_colour(&self) {
        let mut colour_mixer = ColourMixer::new();
        for (colour, parts) in self.series_paint_components.iter_colour_components() {
            colour_mixer.add(&colour, parts)
        }
        for (colour, parts) in self.mixed_paints.components().iter_colour_components() {
            colour_mixer.add(&colour, parts)
        }
        if let Some(ref colour) = colour_mixer.get_colour() {
            self.colour_match_area.set_mixed_colour(Some(colour));
            self.cads.set_colour(Some(colour));
        } else {
            self.colour_match_area.set_mixed_colour(None);
            self.cads.set_colour(None);
        }
        self.set_button_sensitivities();
    }

    fn cancel_current_mixture(&self) {
        self.mixed_paint_notes.set_text("");
        self.set_target_colour(None);
        self.next_name_label.set_text("#00?:");
        self.series_paint_components.reset_all_parts_to_zero();
        self.mixed_paints.components().reset_all_parts_to_zero();
        self.set_button_sensitivities();
    }

    fn simplify_parts(&self) {
        let mut gcd = self.series_paint_components.get_gcd();
        gcd = gcd.gcd(&self.mixed_paints.components().get_gcd());
        self.series_paint_components.divide_all_parts_by(gcd);
        self.mixed_paints.components().divide_all_parts_by(gcd);
    }

    fn pango_markup_chunks(&self) -> Vec<String> {
        let series_paints_used = self.mixed_paints.series_paints_used();

        if series_paints_used.len() == 0 {
            return vec![escape_str_attribute("Empty Mix/Match Description").to_string()];
        }

        let mut text = format!("<b>{}</b> ", escape_str_attribute("Mix/Match Description:"));
        text += &format!("{}\n", Local::now().format("%X: %A %x"));
        if let Some(notes) = self.notes.get_text() {
            if text.len() > 0 {
                text += &format!("\n{}\n", notes);
            }
        };
        let mut chunks = vec![text];

        let mut text = format!("<b>{}</b>\n\n", escape_str_attribute("Paint Colours:"));
        for series_paint in series_paints_used.iter() {
            text += &format!(
                "<span background=\"{}\">\t</span> ",
                series_paint.rgb().pango_string()
            );
            text += &format!("{}", escape_str_attribute(&series_paint.name()));
            if series_paint.notes().len() > 0 {
                text += &format!(" {}\n", escape_str_attribute(&series_paint.notes()));
            } else {
                text += "\n";
            }
        }
        chunks.push(text);

        let mut text = format!("<b>{}</b>\n\n", escape_str_attribute("Mixed Colours:"));
        for mixed_paint in self.mixed_paints.get_paints().iter() {
            text += &format!(
                "<span background=\"{}\">\t</span> ",
                mixed_paint.rgb().pango_string()
            );
            text += &format!(
                "<span background=\"{}\">\t</span> ",
                mixed_paint.monotone_rgb().pango_string()
            );
            text += &format!(
                "<span background=\"{}\">\t</span> ",
                mixed_paint.max_chroma_rgb().pango_string()
            );
            text += &format!("{}", escape_str_attribute(&mixed_paint.name()));
            if mixed_paint.notes().len() > 0 {
                text += &format!(" {}\n", escape_str_attribute(&mixed_paint.notes()));
            } else {
                text += "\n";
            };
            if let Some(matched_colour) = mixed_paint.matched_colour() {
                text += &format!(
                    "<span background=\"{}\">\t</span> ",
                    matched_colour.rgb().pango_string()
                );
                text += &format!(
                    "<span background=\"{}\">\t</span> ",
                    matched_colour.monotone_rgb().pango_string()
                );
                text += &format!(
                    "<span background=\"{}\">\t</span> Matched Colour\n",
                    matched_colour.max_chroma_rgb().pango_string()
                );
            };
            for component in mixed_paint.components().iter() {
                text += &format!("{:7}: ", component.parts);
                text += &format!(
                    "<span background=\"{}\">\t</span> ",
                    component.paint.rgb().pango_string()
                );
                text += &format!("{}\n", escape_str_attribute(&component.paint.name()));
            }
            chunks.push(text);
            text = "".to_string();
        }

        chunks
    }
}

impl_widget_wrapper!(vbox: gtk::Box, PaintMixerCore<A, C, MC>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            MC: MixerConfig + 'static,
);

pub type PaintMixer<A, C, MC> = Rc<PaintMixerCore<A, C, MC>>;

impl<A, C, MC> PaintMixerInterface<A, C, MC> for PaintMixer<A, C, MC>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    MC: MixerConfig + 'static,
{
    fn create(
        series_paint_data_path: &Path,
        paint_standards_data_path: Option<&Path>,
    ) -> PaintMixer<A, C, MC> {
        assert!(
            paint_standards_data_path.is_none() || MC::mixing_mode() == MixingMode::MatchTarget
        );
        let mut view_attr_wheels: Vec<MixerHueAttrWheel<A, C>> = Vec::new();
        for attr in A::scalar_attributes().iter() {
            view_attr_wheels.push(MixerHueAttrWheel::<A, C>::create(*attr));
        }
        let o_paint_standards_manager = if let Some(path) = paint_standards_data_path {
            Some(PaintStandardManager::<A, C>::create(path))
        } else {
            None
        };
        let paint_mixer = Rc::new(PaintMixerCore::<A, C, MC> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
            cads: A::create(),
            hue_attr_wheels: view_attr_wheels,
            colour_match_area: ColourMatchArea::create(MC::mixing_mode()),
            series_paint_components: SeriesPaintComponentBox::<A, C>::create_with(4, true),
            mixed_paints: MixedPaintCollectionWidget::<A, C>::create(MC::mixing_mode()),
            notes: gtk::Entry::new(),
            next_name_label: gtk::Label::new(Some("#???:")),
            mixed_paint_notes: gtk::Entry::new(),
            // Buttons
            print_report_btn: gtk::Button::new(),
            new_mixture_btn: gtk::Button::new_with_label("New"),
            accept_mixture_btn: gtk::Button::new_with_label("Accept"),
            cancel_btn: gtk::Button::new_with_label("Cancel"),
            reset_parts_btn: gtk::Button::new_with_label("Reset"),
            remove_unused_btn: gtk::Button::new_with_label("Remove Unused Paints"),
            simplify_parts_btn: gtk::Button::new_with_label("Simplify Parts"),
            // Managers
            series_paint_manager: SeriesPaintManager::<A, C>::create(series_paint_data_path),
            o_paint_standards_manager: o_paint_standards_manager,
            phantom: PhantomData,
        });

        // TODO: Consider redoing this when Toolbar bug fixed.
        //let toolbar = gtk::Toolbar::new();
        //let paint_mixer.print_report_btn = gtk::ToolButton::new_from_stock("gtk-print");
        //paint_mixer.print_report_btn.set_tooltip_text("Print a report of the mixtures and paints used");
        //toolbar.insert(&paint_mixer.print_report_btn.clone(), 1);
        //toolbar.insert(&paint_mixer.series_paint_manager.tool_button(), 2);
        //toolbar.show_all();
        //paint_mixer.vbox.pack_start(&toolbar, false, false, 0);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&paint_mixer.print_report_btn.clone(), false, true, 2);
        hbox.pack_start(&paint_mixer.series_paint_manager.button(), false, true, 2);
        if let Some(ref paint_standards_manager) = paint_mixer.o_paint_standards_manager {
            hbox.pack_start(&paint_standards_manager.button(), false, true, 2);
        };
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
        notebook.set_scrollable(true);
        notebook.popup_enable();

        let hpaned = gtk::Paned::new(gtk::Orientation::Horizontal);
        hpaned.pack1(&notebook, true, true);
        hpaned.pack2(&vbox, true, true);
        hpaned.set_position_from_recollections("paint_mixer_horizontal", 200);
        paint_mixer.vbox.pack_start(&hpaned, true, true, 0);

        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        paint_mixer.vbox.pack_start(&button_box, false, false, 0);
        if MC::mixing_mode() == MixingMode::MatchTarget {
            button_box.pack_start(&paint_mixer.new_mixture_btn, true, true, 0);
            button_box.pack_start(&paint_mixer.accept_mixture_btn, true, true, 0);
            button_box.pack_start(&paint_mixer.cancel_btn, true, true, 0);
        } else {
            button_box.pack_start(&paint_mixer.accept_mixture_btn, true, true, 0);
        };
        button_box.pack_start(&paint_mixer.simplify_parts_btn, true, true, 0);
        button_box.pack_start(&paint_mixer.reset_parts_btn, true, true, 0);
        button_box.pack_start(&paint_mixer.remove_unused_btn, true, true, 0);

        let frame = gtk::Frame::new(Some("Paints"));
        frame.add(&paint_mixer.series_paint_components.pwo());
        paint_mixer.vbox.pack_start(&frame, true, true, 0);

        let frame = gtk::Frame::new(Some("MixedPaints"));
        frame.add(&paint_mixer.mixed_paints.pwo());
        paint_mixer.vbox.pack_start(&frame, true, true, 0);

        paint_mixer
            .print_report_btn
            .set_image(&mixtures_print_xpm::mixtures_print_image(24));
        paint_mixer
            .print_report_btn
            .set_tooltip_text("Print a report of the mixtures and paints used");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.print_report_btn.connect_clicked(move |_| {
            if let Err(ref err) =
                paint_mixer_c.print_markup_chunks(paint_mixer_c.pango_markup_chunks())
            {
                paint_mixer_c.report_error("Failure", err);
            };
        });

        if MC::mixing_mode() == MixingMode::MatchTarget {
            paint_mixer
                .new_mixture_btn
                .set_tooltip_text("Start mixing a new colour.");
            let paint_mixer_c = paint_mixer.clone();
            paint_mixer.new_mixture_btn.connect_clicked(move |_| {
                let dialog = NewTargetColourDialog::<A>::create(&paint_mixer_c);
                if let Some((ref notes, ref colour)) = dialog.get_new_target() {
                    paint_mixer_c.start_new_mixture(Some(&notes), Some(&colour))
                }
            });

            paint_mixer
                .cancel_btn
                .set_tooltip_text("Cancel the current mixture.");
            let paint_mixer_c = paint_mixer.clone();
            paint_mixer
                .cancel_btn
                .connect_clicked(move |_| paint_mixer_c.cancel_current_mixture());
        };

        paint_mixer.accept_mixture_btn.set_tooltip_text(
            "Accept the current mixture and add it to the list of mixed colours.",
        );
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .accept_mixture_btn
            .connect_clicked(move |_| paint_mixer_c.accept_new_mixture());

        paint_mixer
            .simplify_parts_btn
            .set_tooltip_text("Divide all paints' parts by their greatest common denominator.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .simplify_parts_btn
            .connect_clicked(move |_| paint_mixer_c.simplify_parts());

        paint_mixer
            .reset_parts_btn
            .set_tooltip_text("Reset parts of all paints in mixing part to zero.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.reset_parts_btn.connect_clicked(move |_| {
            paint_mixer_c
                .series_paint_components
                .reset_all_parts_to_zero();
            paint_mixer_c
                .mixed_paints
                .components()
                .reset_all_parts_to_zero();
            paint_mixer_c.set_button_sensitivities();
        });

        // TODO: be more sophisticated about removing series paints
        paint_mixer
            .remove_unused_btn
            .set_tooltip_text("Remove paints not being used from the mixing area.");
        let paint_mixer_c = paint_mixer.clone();
        paint_mixer.remove_unused_btn.connect_clicked(move |_| {
            paint_mixer_c.remove_unused_paints_from_mixing_area();
        });

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .series_paint_components
            .connect_contributions_changed(move || {
                paint_mixer_c.update_mixed_colour();
            });

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .mixed_paints
            .components()
            .connect_contributions_changed(move || {
                paint_mixer_c.update_mixed_colour();
            });

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .series_paint_components
            .connect_removal_requested(move |paint| {
                paint_mixer_c.handle_series_paint_removal_request(paint);
            });

        let mixed_paint_components = paint_mixer.mixed_paints.components();
        paint_mixer
            .mixed_paints
            .components()
            .connect_removal_requested(move |paint| {
                mixed_paint_components.remove_paint(paint);
            });

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .series_paint_manager
            .connect_add_paint(move |paint| paint_mixer_c.add_series_paint(paint));

        let paint_mixer_c = paint_mixer.clone();
        paint_mixer
            .mixed_paints
            .connect_remove_paint(move |paint| paint_mixer_c.remove_mixed_paint(paint));

        if let Some(ref paint_standards_manager) = paint_mixer.o_paint_standards_manager {
            let paint_mixer_c = paint_mixer.clone();
            paint_standards_manager.connect_set_target_from(move |paint| {
                let paint_notes = paint.notes();
                let notes = if paint_notes.len() > 0 {
                    format!("{} ({})", paint.name(), paint_notes)
                } else {
                    paint.name()
                };
                let colour = paint.colour();
                paint_mixer_c.start_new_mixture(Some(&notes), Some(&colour));
            });
        };

        paint_mixer.set_button_sensitivities();

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
