// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::path::Path;
use std::rc::Rc;

use pw_gix::{
    gdk_pixbuf::Pixbuf,
    gtk::{self, prelude::*},
    gtkx::window::*,
    wrapper::*,
};

use crate::basic_paint::*;
use crate::colln_paint::binder::*;
use crate::colln_paint::collection::*;
pub use crate::colln_paint::display::*;
use crate::colln_paint::editor::*;
use crate::colln_paint::*;
use crate::colour::*;
use crate::icons::series_paint_xpm::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
pub struct PaintSeriesId {
    manufacturer: String,
    series_name: String,
}

impl PaintSeriesId {
    pub fn manufacturer(&self) -> String {
        self.manufacturer.clone()
    }

    pub fn series_name(&self) -> String {
        self.series_name.clone()
    }
}

impl CollnIdInterface for PaintSeriesId {
    fn new(colln_name: &str, colln_owner: &str) -> PaintSeriesId {
        PaintSeriesId {
            manufacturer: colln_owner.to_string(),
            series_name: colln_name.to_string(),
        }
    }

    fn colln_name_label() -> String {
        "Series:".to_string()
    }

    fn colln_owner_label() -> String {
        "Manufacturer:".to_string()
    }

    fn paint_select_label() -> String {
        "Add to Mixer".to_string()
    }

    fn paint_select_tooltip_text() -> String {
        "Add this paint to the paint mixing area.".to_string()
    }

    fn recollection_name_for(item_name: &str) -> String {
        format!("series_paint::{}", item_name)
    }

    fn colln_load_image(size: i32) -> gtk::Image {
        series_paint_load_image(size)
    }

    fn colln_name(&self) -> String {
        self.series_name.clone()
    }

    fn colln_owner(&self) -> String {
        self.manufacturer.clone()
    }
}

pub type SeriesPaint<C> = CollnPaint<C, PaintSeriesId>;
pub type SeriesPaintColln<C> = CollnPaintColln<C, PaintSeriesId>;
pub type SeriesPaintCollnSpec<C> = PaintCollnSpec<C, PaintSeriesId>;

pub type SeriesPaintCollnBinder<A, C> = CollnPaintCollnBinder<A, C, PaintSeriesId>;
pub type SeriesPaintDisplayDialog<A, C> = CollnPaintDisplayDialog<A, C, PaintSeriesId>;
pub type SeriesPaintEditor<A, C> = CollnPaintEditor<A, C, PaintSeriesId>;

const TOOLTIP_TEXT: &str = "Open the Series Paint Manager.
This enables paint to be added to the mixer.";

pub struct SeriesPaintManagerCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    window: gtk::Window,
    binder: SeriesPaintCollnBinder<A, C>,
}

impl<A, C> SeriesPaintManagerCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    pub fn set_icon(&self, icon: &Pixbuf) {
        self.window.set_icon(Some(icon))
    }

    pub fn set_initiate_select_ok(&self, value: bool) {
        self.binder.set_initiate_select_ok(value);
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        self.binder.set_target_colour(ocolour)
    }

    pub fn connect_add_paint<F: 'static + Fn(&SeriesPaint<C>)>(&self, callback: F) {
        self.binder.connect_paint_selected(callback)
    }
}

pub type SeriesPaintManager<A, C> = Rc<SeriesPaintManagerCore<A, C>>;

pub trait SeriesPaintManagerInterface<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C>;
    fn button(&self) -> gtk::Button;
    fn tool_button(&self) -> gtk::ToolButton;
}

impl<A, C> SeriesPaintManagerInterface<A, C> for SeriesPaintManager<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> SeriesPaintManager<A, C> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_geometry_from_recollections("series_paint_manager", (600, 200));
        window.set_destroy_with_parent(true);
        window.set_title("Series Paint Manager");
        window.connect_delete_event(move |w, _| {
            w.hide_on_delete();
            gtk::Inhibit(true)
        });
        let binder = SeriesPaintCollnBinder::<A, C>::create(data_path);
        binder.set_initiate_select_ok(true);
        window.add(&binder.pwo());

        let spm = Rc::new(SeriesPaintManagerCore::<A, C> { window, binder });

        spm
    }

    fn button(&self) -> gtk::Button {
        let button = gtk::Button::new();
        button.set_tooltip_text(Some(TOOLTIP_TEXT));
        button.set_image(Some(&series_paint_image(24)));
        let spm_c = self.clone();
        button.connect_clicked(move |_| spm_c.window.present());
        button
    }

    fn tool_button(&self) -> gtk::ToolButton {
        let tool_button =
            gtk::ToolButton::new(Some(&series_paint_image(24)), Some("Series Paint Manager"));
        tool_button.set_tooltip_text(Some(TOOLTIP_TEXT));
        let spm_c = self.clone();
        tool_button.connect_clicked(move |_| spm_c.window.present());
        tool_button
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
