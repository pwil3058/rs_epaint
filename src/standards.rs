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

use std::path::Path;
use std::rc::Rc;

use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::window::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use basic_paint::editor::*;
use colln_paint::*;
use colln_paint::binder::*;
use colln_paint::collection::*;
pub use colln_paint::display::*;
use icons::paint_standard_xpms::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Default, Hash)]
pub struct PaintStandardId {
    sponsor: String,
    standard: String,
}

impl PaintStandardId {
    pub fn sponsor(&self) -> String {
        self.sponsor.clone()
    }

    pub fn standard(&self) -> String {
        self.standard.clone()
    }
}

impl CollnIdInterface for PaintStandardId {
    fn new(colln_name: &str, colln_owner: &str) -> PaintStandardId {
        PaintStandardId{
            sponsor: colln_owner.to_string(),
            standard: colln_name.to_string(),
        }
    }

    fn colln_name_label() -> String {
        "Standard:".to_string()
    }

    fn colln_owner_label() -> String {
        "Sponsor:".to_string()
    }

    fn paint_select_label() -> String {
        "Set Target".to_string()
    }

    fn paint_select_tooltip_text() -> String {
        "Set the target colour in the mixing area from this paint.".to_string()
    }

    fn recollection_name_for(item_name: &str) -> String {
        format!("paint_standard::{}", item_name)
    }

    fn colln_load_image(size: i32) -> gtk::Image {
        paint_standard_load_image(size)
    }

    fn colln_name(&self) -> String {
        self.standard.clone()
    }

    fn colln_owner(&self) -> String {
        self.sponsor.clone()
    }
}

pub type PaintStandard<C> = CollnPaint<C, PaintStandardId>;
pub type PaintStandardColln<C> = CollnPaintColln<C, PaintStandardId>;
pub type PaintStandardCollnSpec<C> = PaintCollnSpec<C, PaintStandardId>;

pub type PaintStandardCollnBinder<A, C> = CollnPaintCollnBinder<A, C, PaintStandardId>;
pub type PaintStandardDisplayDialog<A, C> = CollnPaintDisplayDialog<A, C, PaintStandardId>;
pub type PaintStandardEditor<A, C> = BasicPaintEditor<A, C, PaintStandardId>;

const TOOLTIP_TEXT: &str =
"Open the Series Paint Manager.
This enables paint to be added to the mixer.";

pub struct PaintStandardManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    window: gtk::Window,
    binder: PaintStandardCollnBinder<A, C>,
}

impl<A,C> PaintStandardManagerCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    pub fn set_icon(&self, icon: &Pixbuf) {
        self.window.set_icon(Some(icon))
    }

    pub fn set_initiate_select_ok(&self, value: bool) {
        self.binder.set_initiate_select_ok(value);
    }

    pub fn connect_set_target_from<F: 'static + Fn(&PaintStandard<C>)>(&self, callback: F) {
        self.binder.connect_paint_selected(callback)
    }
}

pub type PaintStandardManager<A, C> = Rc<PaintStandardManagerCore<A, C>>;

pub trait PaintStandardManagerInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> PaintStandardManager<A, C>;
    fn button(&self) -> gtk::Button;
    fn tool_button(&self) -> gtk::ToolButton;
}


impl<A, C> PaintStandardManagerInterface<A, C> for PaintStandardManager<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(data_path: &Path) -> PaintStandardManager<A, C> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_geometry_from_recollections("paint_standards_manager", (600, 200));
        window.set_destroy_with_parent(true);
        window.set_title("Paint Standards Manager");
        window.connect_delete_event(
            move |w,_| {w.hide_on_delete(); gtk::Inhibit(true)}
        );
        let binder = PaintStandardCollnBinder::<A, C>::create(data_path);
        binder.set_initiate_select_ok(true);
        window.add(&binder.pwo());

        let spm = Rc::new(
            PaintStandardManagerCore::<A, C>{window, binder}
        );

        spm
    }

    fn button(&self) -> gtk::Button {
        let button = gtk::Button::new();
        button.set_tooltip_text(Some(TOOLTIP_TEXT));
        button.set_image(&paint_standard_image(24));
        let spm_c = self.clone();
        button.connect_clicked(
            move |_| spm_c.window.present()
        );
        button
    }

    fn tool_button(&self) -> gtk::ToolButton {
        let tool_button = gtk::ToolButton::new(Some(&paint_standard_image(24)), Some("Paint Standards Manager"));
        tool_button.set_tooltip_text(Some(TOOLTIP_TEXT));
        let spm_c = self.clone();
        tool_button.connect_clicked(
            move |_| spm_c.window.present()
        );
        tool_button
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

}
