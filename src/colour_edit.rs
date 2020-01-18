// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cairo;
use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;

use normalised_angles::Degrees;

use pw_gix::cairox::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::entry::*;
use pw_gix::gtkx::menu::*;

pub use pw_gix::wrapper::*;

use crate::basic_paint::*;
use crate::colour::*;

#[derive(Debug, PartialEq, Clone, Copy)]
enum DeltaSize {
    Small,
    Normal,
    Large,
}

impl DeltaSize {
    fn for_value(&self) -> f64 {
        match *self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_chroma(&self) -> f64 {
        match *self {
            DeltaSize::Small => 0.0025,
            DeltaSize::Normal => 0.005,
            DeltaSize::Large => 0.01,
        }
    }

    fn for_hue(&self) -> Degrees<f64> {
        match *self {
            DeltaSize::Small => 0.5.into(),
            DeltaSize::Normal => 1.0.into(),
            DeltaSize::Large => 5.0.into(),
        }
    }
}

struct Sample {
    pix_buf: gdk_pixbuf::Pixbuf,
    position: Point,
}

pub trait ColourEditorInterface {
    fn create(extra_buttons: &Vec<gtk::Button>) -> Self;
}

#[derive(PWO, Wrapper)]
pub struct ColourEditorCore<A>
where
    A: ColourAttributesInterface + 'static,
{
    vbox: gtk::Box,
    rgb_manipulator: RefCell<RGBManipulator>,
    cads: Rc<A>,
    rgb_entry: RGBHexEntryBox,
    drawing_area: gtk::DrawingArea,
    incr_value_btn: gtk::Button,
    decr_value_btn: gtk::Button,
    hue_left_btn: gtk::Button,
    hue_right_btn: gtk::Button,
    decr_greyness_btn: gtk::Button,
    incr_greyness_btn: gtk::Button,
    decr_chroma_btn: gtk::Button,
    incr_chroma_btn: gtk::Button,
    popup_menu: WrappedMenu,
    samples: RefCell<Vec<Sample>>,
    delta_size: Cell<DeltaSize>,
    popup_menu_position: Cell<Point>,
    auto_match_btn: gtk::Button,
    auto_match_on_paste_btn: gtk::CheckButton,
    colour_changed_callbacks: RefCell<Vec<Box<dyn Fn(&Colour)>>>,
}

impl<A> ColourEditorCore<A>
where
    A: ColourAttributesInterface + 'static,
{
    pub fn set_rgb(&self, rgb: RGB) -> Colour {
        let colour = Colour::from(rgb);
        self.rgb_entry.set_rgb(rgb);
        self.rgb_manipulator.borrow_mut().set_rgb(rgb);
        self.cads.set_colour(Some(&colour));
        self.incr_value_btn
            .set_widget_colour_rgb(rgb * 0.8 + RGB::WHITE * 0.2);
        self.decr_value_btn.set_widget_colour_rgb(rgb * 0.8);
        if colour.is_grey() {
            self.incr_greyness_btn.set_widget_colour_rgb(rgb);
            self.decr_greyness_btn.set_widget_colour_rgb(rgb);
            self.incr_chroma_btn.set_widget_colour_rgb(rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(rgb);
            self.hue_left_btn.set_widget_colour_rgb(rgb);
            self.hue_right_btn.set_widget_colour_rgb(rgb);
        } else {
            let low_chroma_rgb = rgb * 0.8 + colour.monochrome_rgb() * 0.2;
            let high_chroma_rgb = rgb * 0.8 + colour.max_chroma_rgb() * 0.2;
            self.incr_greyness_btn.set_widget_colour_rgb(low_chroma_rgb);
            self.decr_greyness_btn
                .set_widget_colour_rgb(high_chroma_rgb);
            self.incr_chroma_btn.set_widget_colour_rgb(high_chroma_rgb);
            self.decr_chroma_btn.set_widget_colour_rgb(low_chroma_rgb);

            self.hue_left_btn
                .set_widget_colour_rgb(rgb.components_rotated(Degrees::DEG_30));
            self.hue_right_btn
                .set_widget_colour_rgb(rgb.components_rotated(-Degrees::DEG_30));
        }
        self.drawing_area.queue_draw();
        colour
    }

    pub fn get_rgb(&self) -> RGB {
        self.rgb_manipulator.borrow().rgb()
    }

    pub fn get_colour(&self) -> Colour {
        self.rgb_manipulator.borrow().rgb().into()
    }

    fn set_rgb_and_inform(&self, rgb: RGB) {
        let colour = self.set_rgb(rgb);
        for callback in self.colour_changed_callbacks.borrow().iter() {
            callback(&colour);
        }
    }

    fn draw(&self, _drawing_area: &gtk::DrawingArea, cairo_context: &cairo::Context) {
        let rgb = self.rgb_manipulator.borrow().rgb();
        cairo_context.set_source_colour_rgb(rgb);
        cairo_context.paint();
        for sample in self.samples.borrow().iter() {
            cairo_context.set_source_pixbuf_at(&sample.pix_buf, sample.position);
            cairo_context.set_line_width(0.0);
            cairo_context.paint();
        }
    }

    fn auto_match_samples(&self) {
        let mut red: u64 = 0;
        let mut green: u64 = 0;
        let mut blue: u64 = 0;
        let mut npixels: u32 = 0;
        for sample in self.samples.borrow().iter() {
            assert_eq!(sample.pix_buf.get_bits_per_sample(), 8);
            let nc = sample.pix_buf.get_n_channels();
            let rs = sample.pix_buf.get_rowstride();
            let width = sample.pix_buf.get_width();
            let n_rows = sample.pix_buf.get_height();
            unsafe {
                let data = sample.pix_buf.get_pixels();
                for row_num in 0..n_rows {
                    let row_start = row_num * rs;
                    for j in 0..width {
                        let offset = (row_start + j * nc) as usize;
                        red += data[offset] as u64;
                        green += data[offset + 1] as u64;
                        blue += data[offset + 2] as u64;
                    }
                }
            }
            npixels += (width * n_rows) as u32;
        }
        if npixels > 0 {
            let divisor = (npixels * 255) as f64;
            let array: [f64; 3] = [
                red as f64 / divisor,
                green as f64 / divisor,
                blue as f64 / divisor,
            ];
            self.set_rgb_and_inform(array.into());
        }
    }

    pub fn reset(&self) {
        self.samples.borrow_mut().clear();
        self.set_rgb_and_inform(RGB::WHITE * 0.5);
    }

    pub fn connect_colour_changed<F: 'static + Fn(&Colour)>(&self, callback: F) {
        self.colour_changed_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }
}

impl<A> ColourEditorInterface for Rc<ColourEditorCore<A>>
where
    A: ColourAttributesInterface + 'static,
{
    fn create(extra_buttons: &Vec<gtk::Button>) -> Self {
        let ced = Rc::new(ColourEditorCore::<A> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            rgb_manipulator: RefCell::new(RGBManipulator::new(false)),
            cads: A::create(),
            rgb_entry: RGBHexEntryBox::create(),
            drawing_area: gtk::DrawingArea::new(),
            incr_value_btn: gtk::Button::new_with_label("Value++"),
            decr_value_btn: gtk::Button::new_with_label("Value--"),
            hue_left_btn: gtk::Button::new_with_label("<"),
            hue_right_btn: gtk::Button::new_with_label(">"),
            decr_greyness_btn: gtk::Button::new_with_label("Greyness--"),
            incr_greyness_btn: gtk::Button::new_with_label("Greyness++"),
            decr_chroma_btn: gtk::Button::new_with_label("Chroma--"),
            incr_chroma_btn: gtk::Button::new_with_label("Chroma++"),
            popup_menu: WrappedMenu::new(&vec![]),
            samples: RefCell::new(Vec::new()),
            delta_size: Cell::new(DeltaSize::Normal),
            auto_match_btn: gtk::Button::new_with_label("Auto Match"),
            auto_match_on_paste_btn: gtk::CheckButton::new_with_label("On Paste?"),
            popup_menu_position: Cell::new(Point(0.0, 0.0)),
            colour_changed_callbacks: RefCell::new(Vec::new()),
        });

        let events = gdk::EventMask::BUTTON_PRESS_MASK;
        ced.drawing_area.add_events(events);
        ced.drawing_area.set_size_request(200, 200);

        ced.vbox.pack_start(&ced.rgb_entry.pwo(), false, false, 0);
        ced.vbox.pack_start(&ced.cads.pwo(), false, false, 0);

        ced.vbox
            .pack_start(&ced.incr_value_btn.clone(), false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&ced.hue_left_btn.clone(), false, false, 0);
        hbox.pack_start(&ced.drawing_area.clone(), true, true, 0);
        hbox.pack_start(&ced.hue_right_btn.clone(), false, false, 0);
        ced.vbox.pack_start(&hbox, true, true, 0);

        ced.vbox
            .pack_start(&ced.decr_value_btn.clone(), false, false, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        if A::scalar_attributes().contains(&ScalarAttribute::Greyness) {
            hbox.pack_start(&ced.decr_greyness_btn.clone(), true, true, 0);
            hbox.pack_start(&ced.incr_greyness_btn.clone(), true, true, 0);
        } else {
            hbox.pack_start(&ced.decr_chroma_btn.clone(), true, true, 0);
            hbox.pack_start(&ced.incr_chroma_btn.clone(), true, true, 0);
        };
        ced.vbox.pack_start(&hbox, false, false, 0);

        let bbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        for button in extra_buttons.iter() {
            bbox.pack_start(&button.clone(), true, true, 0);
        }
        bbox.pack_start(&ced.auto_match_btn.clone(), true, true, 0);
        bbox.pack_start(&ced.auto_match_on_paste_btn.clone(), false, false, 0);
        ced.vbox.pack_start(&bbox, false, false, 0);

        ced.vbox.show_all();

        let events = gdk::EventMask::KEY_PRESS_MASK
            | gdk::EventMask::KEY_RELEASE_MASK
            | gdk::EventMask::ENTER_NOTIFY_MASK;
        ced.vbox.add_events(events);
        ced.vbox.set_receives_default(true);
        let ced_c = ced.clone();
        ced.vbox.connect_key_press_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L || key == gdk::enums::key::Shift_R {
                ced_c.delta_size.set(DeltaSize::Large);
            } else if key == gdk::enums::key::Control_L || key == gdk::enums::key::Control_R {
                ced_c.delta_size.set(DeltaSize::Small);
            };
            gtk::Inhibit(false)
        });
        let ced_c = ced.clone();
        ced.vbox.connect_key_release_event(move |_, event| {
            let key = event.get_keyval();
            if key == gdk::enums::key::Shift_L
                || key == gdk::enums::key::Shift_R
                || key == gdk::enums::key::Control_L
                || key == gdk::enums::key::Control_R
            {
                ced_c.delta_size.set(DeltaSize::Normal);
            };
            gtk::Inhibit(false)
        });
        let ced_c = ced.clone();
        ced.vbox.connect_enter_notify_event(move |_, _| {
            ced_c.delta_size.set(DeltaSize::Normal);
            gtk::Inhibit(false)
        });

        if A::scalar_attributes().contains(&ScalarAttribute::Greyness) {
            let ced_c = ced.clone();
            ced.incr_greyness_btn.connect_clicked(move |_| {
                let changed = ced_c
                    .rgb_manipulator
                    .borrow_mut()
                    .decr_chroma(ced_c.delta_size.get().for_chroma());
                if changed {
                    let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                    ced_c.set_rgb_and_inform(new_rgb);
                };
            });
            let ced_c = ced.clone();
            ced.decr_greyness_btn.connect_clicked(move |_| {
                let changed = ced_c
                    .rgb_manipulator
                    .borrow_mut()
                    .incr_chroma(ced_c.delta_size.get().for_chroma());
                if changed {
                    let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                    ced_c.set_rgb_and_inform(new_rgb);
                };
            });
        } else {
            let ced_c = ced.clone();
            ced.decr_chroma_btn.connect_clicked(move |_| {
                let changed = ced_c
                    .rgb_manipulator
                    .borrow_mut()
                    .decr_chroma(ced_c.delta_size.get().for_chroma());
                if changed {
                    let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                    ced_c.set_rgb_and_inform(new_rgb);
                };
            });
            let ced_c = ced.clone();
            ced.incr_chroma_btn.connect_clicked(move |_| {
                let changed = ced_c
                    .rgb_manipulator
                    .borrow_mut()
                    .incr_chroma(ced_c.delta_size.get().for_chroma());
                if changed {
                    let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                    ced_c.set_rgb_and_inform(new_rgb);
                };
            });
        }

        let ced_c = ced.clone();
        ced.decr_value_btn.connect_clicked(move |_| {
            let changed = ced_c
                .rgb_manipulator
                .borrow_mut()
                .decr_value(ced_c.delta_size.get().for_value());
            if changed {
                let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(new_rgb);
            };
        });
        let ced_c = ced.clone();
        ced.incr_value_btn.connect_clicked(move |_| {
            let changed = ced_c
                .rgb_manipulator
                .borrow_mut()
                .incr_value(ced_c.delta_size.get().for_value());
            if changed {
                let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(new_rgb);
            };
        });

        let ced_c = ced.clone();
        ced.hue_left_btn.connect_clicked(move |_| {
            let changed = ced_c
                .rgb_manipulator
                .borrow_mut()
                .rotate(ced_c.delta_size.get().for_hue());
            if changed {
                let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(new_rgb);
            };
        });
        let ced_c = ced.clone();
        ced.hue_right_btn.connect_clicked(move |_| {
            let changed = ced_c
                .rgb_manipulator
                .borrow_mut()
                .rotate(-ced_c.delta_size.get().for_hue());
            if changed {
                let new_rgb = ced_c.rgb_manipulator.borrow().rgb();
                ced_c.set_rgb_and_inform(new_rgb);
            };
        });

        let ced_c = ced.clone();
        ced.rgb_entry.connect_value_changed(move |rgb| {
            ced_c.set_rgb_and_inform(rgb);
        });

        let ced_c = ced.clone();
        ced.auto_match_btn
            .connect_clicked(move |_| ced_c.auto_match_samples());

        let ced_c = ced.clone();
        ced.drawing_area.connect_draw(move |da, cctx| {
            ced_c.draw(da, cctx);
            gtk::Inhibit(true)
        });

        let ced_c = ced.clone();
        ced.popup_menu
            .append_item(
                "paste",
                "Paste Sample",
                "Paste image sample from the clipboard at this position",
            )
            .connect_activate(move |_| {
                let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                if let Some(pixbuf) = cbd.wait_for_image() {
                    let sample = Sample {
                        pix_buf: pixbuf,
                        position: ced_c.popup_menu_position.get(),
                    };
                    ced_c.samples.borrow_mut().push(sample);
                    if ced_c.auto_match_on_paste_btn.get_active() {
                        ced_c.auto_match_samples();
                    } else {
                        ced_c.drawing_area.queue_draw();
                    };
                    ced_c.auto_match_btn.set_sensitive(true);
                } else {
                    ced_c.inform_user("No image data on clipboard.", None);
                }
            });

        let ced_c = ced.clone();
        ced.popup_menu
            .append_item(
                "remove",
                "Remove Sample(s)",
                "Remove all image samples from the sample area",
            )
            .connect_activate(move |_| {
                ced_c.samples.borrow_mut().clear();
                ced_c.drawing_area.queue_draw();
                ced_c.auto_match_btn.set_sensitive(false);
            });

        let ced_c = ced.clone();
        ced.drawing_area
            .connect_button_press_event(move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let position = Point::from(event.get_position());
                        let n_samples = ced_c.samples.borrow().len();
                        let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                        ced_c
                            .popup_menu
                            .set_sensitivities(cbd.wait_is_image_available(), &["paste"]);
                        ced_c
                            .popup_menu
                            .set_sensitivities(n_samples > 0, &["remove"]);
                        ced_c.popup_menu_position.set(position);
                        ced_c.popup_menu.popup_at_event(event);
                        return Inhibit(true);
                    }
                }
                Inhibit(false)
            });

        ced.reset();

        ced
    }
}

pub type ColourEditor<A> = Rc<ColourEditorCore<A>>;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
