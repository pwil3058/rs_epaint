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
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::menu::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::wrapper::*;

use basic_paint::*;
use colln_paint::*;
use series_paint::*;
use mixed_paint::*;

use super::*;

pub struct PaintDisplayButtonSpec {
    pub label: String,
    pub tooltip_text: String,
    pub callback: Box<Fn()>
}

static mut NEXT_DIALOG_ID: u32 = 0;

fn get_id_for_dialog() -> u32 {
    let id: u32;
    unsafe {
        id = NEXT_DIALOG_ID;
        NEXT_DIALOG_ID += 1;
    }
    id
}

pub struct PaintDisplayDialogCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    dialog: gtk::Dialog,
    paint: Paint<C>,
    current_target_label: gtk::Label,
    cads: Rc<A>,
    components_view: Option<PaintComponentListView<A, C>>,
    id_no: u32,
    destroy_callbacks: RefCell<Vec<Box<Fn(u32)>>>
}

impl<A, C> PaintDisplayDialogCore<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    pub fn set_transient_for_from<W: gtk::WidgetExt>(&self, widget: &W) {
        self.dialog.set_transient_for_from(widget)
    }

    pub fn show(&self) {
        self.dialog.show()
    }

    pub fn id_no(&self) -> u32 {
        self.id_no
    }

    pub fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if let Some(ref colour) = new_current_target {
            self.current_target_label.set_label("Current Target");
            self.current_target_label.set_widget_colour(&colour);
            self.cads.set_target_colour(Some(&colour));
            if let Some(ref components_view) = self.components_view {
                components_view.set_target_colour(Some(&colour))
            }
        } else {
            self.current_target_label.set_label("");
            self.current_target_label.set_widget_colour(&self.paint.colour());
            self.cads.set_target_colour(None);
            if let Some(ref components_view) = self.components_view {
                components_view.set_target_colour(None)
            }
        };
    }

    pub fn connect_destroy<F: 'static + Fn(u32)>(&self, callback: F) {
        self.destroy_callbacks.borrow_mut().push(Box::new(callback))
    }

    pub fn inform_destroy(&self) {
        for callback in self.destroy_callbacks.borrow().iter() {
            callback(self.id_no);
        }
    }

}

pub type PaintDisplayDialog<A, C> = Rc<PaintDisplayDialogCore<A, C>>;

pub trait PaintDisplayDialogInterface<A, C>
    where   C: CharacteristicsInterface + 'static,
            A: ColourAttributesInterface + 'static
{
    fn create(
        paint: &Paint<C>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> PaintDisplayDialog<A, C>;

    fn series_create(
        paint: &SeriesPaint<C>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> PaintDisplayDialog<A, C> {
        Self::create(&Paint::Series(paint.clone()), current_target, parent, button_specs)
    }

    fn mixed_create(
        paint: &MixedPaint<C>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> PaintDisplayDialog<A, C> {
        Self::create(&Paint::Mixed(paint.clone()), current_target, parent, button_specs)
    }
}

impl<A, C> PaintDisplayDialogInterface<A, C> for PaintDisplayDialog<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(
        paint: &Paint<C>,
        current_target: Option<&Colour>,
        parent: Option<&gtk::Window>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> PaintDisplayDialog<A, C> {
        let title = format!("mcmmtk: {}", paint.name());
        let dialog = gtk::Dialog::new_with_buttons(
            Some(title.as_str()),
            parent,
            gtk::DialogFlags::USE_HEADER_BAR,
            &[]
        );
        if paint.is_series() {
            dialog.set_size_from_recollections("series_paint_display", (60, 330));
        } else {
            dialog.set_size_from_recollections("mixed_paint_display", (60, 330));
        };
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(paint.name().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(paint.notes().as_str());
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        if let Paint::Series(ref series_paint) = *paint {
            let label = gtk::Label::new(series_paint.colln_id().series_name().as_str());
            label.set_widget_colour(&paint.colour());
            vbox.pack_start(&label, false, false, 0);
            let label = gtk::Label::new(series_paint.colln_id().manufacturer().as_str());
            label.set_widget_colour(&paint.colour());
            vbox.pack_start(&label, false, false, 0);
        }
        //
        let current_target_label = gtk::Label::new("");
        current_target_label.set_widget_colour(&paint.colour());
        vbox.pack_start(&current_target_label.clone(), true, true, 0);
        //
        if let Paint::Mixed(ref mixed_paint) = *paint {
            if let Some(colour) = mixed_paint.matched_colour() {
                let current_target_label = gtk::Label::new("Target Colour");
                current_target_label.set_widget_colour(&colour.clone());
                vbox.pack_start(&current_target_label.clone(), true, true, 0);
            }
        }
        //
        content_area.pack_start(&vbox, true, true, 0);
        let cads = A::create();
        cads.set_colour(Some(&paint.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        let characteristics_display = paint.characteristics().gui_display_widget();
        content_area.pack_start(&characteristics_display, false, false, 0);
        let components_view = if let Paint::Mixed(ref mixed_paint) = *paint {
            let cv = PaintComponentListView::<A, C>::create(&mixed_paint.components(), current_target);
            content_area.pack_start(&cv.pwo(), false, false, 0);
            Some(cv)
        } else {
            None
        };
        content_area.show_all();
        for (response_id, spec) in button_specs.iter().enumerate() {
            let button = dialog.add_button(spec.label.as_str(), response_id as i32);
            button.set_tooltip_text(Some(spec.tooltip_text.as_str()));
        };
        dialog.connect_response (
            move |_, r_id| {
                if r_id >= 0 && r_id < button_specs.len() as i32 {
                    (button_specs[r_id as usize].callback)()
                }
            }
        );
        let spd_dialog = Rc::new(
            PaintDisplayDialogCore {
                dialog: dialog,
                paint: paint.clone(),
                current_target_label: current_target_label,
                cads: cads,
                components_view: components_view,
                id_no: get_id_for_dialog(),
                destroy_callbacks: RefCell::new(Vec::new()),
            }
        );
        spd_dialog.set_current_target(current_target);
        let spd_dialog_c = spd_dialog.clone();
        spd_dialog.dialog.connect_destroy(
            move |_| {
                spd_dialog_c.inform_destroy()
            }
        );

        spd_dialog
    }
}

lazy_static! {
    pub static ref PAINT_COMPONENTS_ROW_SPEC: [gtk::Type; 20] =
        [
            gtk::Type::String,          // 0 Name
            gtk::Type::String,          // 1 Notes
            gtk::Type::String,          // 2 Chroma
            gtk::Type::String,          // 3 Greyness
            gtk::Type::String,          // 4 Value
            gtk::Type::String,          // 5 Warmth
            gdk::RGBA::static_type(),   // 6 RGB
            gdk::RGBA::static_type(),   // 7 FG for RGB
            gdk::RGBA::static_type(),   // 8 Monochrome RGB
            gdk::RGBA::static_type(),   // 9 FG for Monochrome RGB
            gdk::RGBA::static_type(),   // 10 Warmth RGB
            gdk::RGBA::static_type(),   // 11 FG for Warmth RGB
            gdk::RGBA::static_type(),   // 12 Hue Colour
            f64::static_type(),         // 13 Hue angle (radians)
            u32::static_type(),         // 14 Paint Index
            gtk::Type::String,          // 15 Parts
            gtk::Type::String,          // 16 Characteristic #1
            gtk::Type::String,          // 17 Characteristic #2
            gtk::Type::String,          // 18 Characteristic #3
            gtk::Type::String,          // 19 Characteristic #4
        ];
}

impl<C> PaintComponent<C>
    where   C: CharacteristicsInterface + 'static,
{
    fn tv_rows(&self, index: u32) -> Vec<gtk::Value> {
        let rgba: gdk::RGBA = self.paint.rgb().into();
        let frgba: gdk::RGBA = self.paint.rgb().best_foreground_rgb().into();
        let mrgba: gdk::RGBA = self.paint.monotone_rgb().into();
        let mfrgba: gdk::RGBA = self.paint.monotone_rgb().best_foreground_rgb().into();
        let wrgba: gdk::RGBA = self.paint.warmth_rgb().into();
        let wfrgba: gdk::RGBA = self.paint.warmth_rgb().best_foreground_rgb().into();
        let hrgba: gdk::RGBA = self.paint.max_chroma_rgb().into();
        let mut rows = vec![
            self.paint.name().to_value(),
            self.paint.notes().to_value(),
            format!("{:5.4}", self.paint.chroma()).to_value(),
            format!("{:5.4}", self.paint.greyness()).to_value(),
            format!("{:5.4}", self.paint.value()).to_value(),
            format!("{:5.4}", self.paint.warmth()).to_value(),
            rgba.to_value(),
            frgba.to_value(),
            mrgba.to_value(),
            mfrgba.to_value(),
            wrgba.to_value(),
            wfrgba.to_value(),
            hrgba.to_value(),
            self.paint.hue().angle().radians().to_value(),
            index.to_value(),
            format!("{:3}", self.parts).to_value(),
        ];
        for row in self.paint.characteristics().tv_rows().iter() {
            rows.push(row.clone());
        };
        rows
    }

}

pub struct PaintComponentListViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    popup_menu: WrappedMenu,
    components: Rc<Vec<PaintComponent<C>>>,
    chosen_paint: RefCell<Option<Paint<C>>>,
    current_target: RefCell<Option<Colour>>,
    paint_dialogs: RefCell<HashMap<u32, PaintDisplayDialog<A, C>>>,
    spec: PhantomData<A>
}

impl<A, C> PaintComponentListViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn get_paint_at(&self, posn: (f64, f64)) -> Option<Paint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let index: u32 = self.list_store.get_value(&iter, 14).get().unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    if index as usize >= self.components.len() {
                        panic!("File: {:?} Line: {:?} outside array bounds", file!(), line!())
                    };
                    let paint = self.components[index as usize].paint.clone();
                    return Some(paint)
                }
            }
        }
        None
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        match ocolour {
            Some(colour) => {
                for dialog in self.paint_dialogs.borrow().values() {
                    dialog.set_current_target(Some(colour));
                };
                *self.current_target.borrow_mut() = Some(colour.clone())
            },
            None => {
                for dialog in self.paint_dialogs.borrow().values() {
                    dialog.set_current_target(None);
                };
                *self.current_target.borrow_mut() = None
            },
        }
    }
}

impl<A, C> WidgetWrapper<gtk::ScrolledWindow> for PaintComponentListViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }
}

pub type PaintComponentListView<A, C> = Rc<PaintComponentListViewCore<A, C>>;

pub trait PaintComponentListViewInterface<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(
        components: &Rc<Vec<PaintComponent<C>>>,
        current_target: Option<&Colour>
    ) -> PaintComponentListView<A, C>;
}

impl<A, C> PaintComponentListViewInterface<A, C> for PaintComponentListView<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
{
    fn create(
        components: &Rc<Vec<PaintComponent<C>>>,
        current_target: Option<&Colour>
    ) -> PaintComponentListView<A, C> {
        let len = 16 + C::tv_row_len();
        let list_store = gtk::ListStore::new(&PAINT_COMPONENTS_ROW_SPEC[0..len]);
        for (index, component) in components.iter().enumerate() {
            list_store.append_row(&component.tv_rows(index as u32));
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let my_current_target = if let Some(colour) = current_target {
            Some(colour.clone())
        } else {
            None
        };

        let pclv = Rc::new(
            PaintComponentListViewCore::<A, C> {
                scrolled_window: gtk::ScrolledWindow::new(None, None),
                list_store: list_store,
                popup_menu: WrappedMenu::new(&vec![]),
                components: components.clone(),
                view: view,
                chosen_paint: RefCell::new(None),
                current_target: RefCell::new(my_current_target),
                paint_dialogs: RefCell::new(HashMap::new()),
                spec: PhantomData,
            }
        );

        pclv.view.append_column(&simple_text_column("Parts", 15, 15, -1, -1, -1, true));
        pclv.view.append_column(&simple_text_column("Name", 0, 0, 6, 7, -1, true));
        pclv.view.append_column(&simple_text_column("Notes", 1, 1, 6, 7, -1, true));
        for col in A::tv_columns() {
            pclv.view.append_column(&col);
        }
        for col in C::tv_columns(16) {
            pclv.view.append_column(&col);
        }

        pclv.view.show_all();

        pclv.scrolled_window.add(&pclv.view.clone());
        pclv.scrolled_window.show_all();

        let pclv_c = pclv.clone();
        pclv.popup_menu.append_item(
            "info",
            "Paint Information",
            "Display this paint's information",
        ).connect_activate(
            move |_| {
                if let Some(ref paint) = *pclv_c.chosen_paint.borrow() {
                    let current_target = pclv_c.current_target.borrow().clone();
                    let target = if let Some(ref colour) = current_target {
                        Some(colour)
                    } else {
                        None
                    };
                    let dialog = PaintDisplayDialog::<A, C>::create(
                        &paint,
                        target,
                        None,
                        vec![]
                    );
                    let pclv_c_c = pclv_c.clone();
                    dialog.connect_destroy(
                        move |id| { pclv_c_c.paint_dialogs.borrow_mut().remove(&id); }
                    );
                    pclv_c.paint_dialogs.borrow_mut().insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            }
        );

        let pclv_c = pclv.clone();
        pclv.view.connect_button_press_event(
            move |_, event| {
                if event.get_event_type() == gdk::EventType::ButtonPress {
                    if event.get_button() == 3 {
                        let o_paint = pclv_c.get_paint_at(event.get_position());
                        pclv_c.popup_menu.set_sensitivities(o_paint.is_some(), &["info"]);
                        *pclv_c.chosen_paint.borrow_mut() = o_paint;
                        // TODO: needs v3_22: pclv_c.menu.popup_at_pointer(None);
                        pclv_c.popup_menu.popup_at_event(event);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
             }
        );

        pclv
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
