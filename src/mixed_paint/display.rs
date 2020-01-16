// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::menu::*;
use pw_gix::gtkx::tree_view_column::*;
use pw_gix::wrapper::*;

use crate::basic_paint::*;
use crate::colour::*;
use crate::dialogue::*;
use crate::series_paint::*;

use super::*;

pub struct MixedPaintDisplayDialogCore<A, C>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: MixedPaint<C>,
    current_target_label: gtk::Label,
    cads: Rc<A>,
    components_view: PaintComponentListView<A, C>,
    id_no: u32,
    destroyed_callbacks: DestroyedCallbacks,
}

pub type MixedPaintDisplayDialog<A, C> = Rc<MixedPaintDisplayDialogCore<A, C>>;

impl<A, C> DialogWrapper for MixedPaintDisplayDialog<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn dialog(&self) -> gtk::Dialog {
        self.dialog.clone()
    }
}

impl<A, C> TrackedDialog for MixedPaintDisplayDialog<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn id_no(&self) -> u32 {
        self.id_no
    }

    fn destroyed_callbacks(&self) -> &DestroyedCallbacks {
        &self.destroyed_callbacks
    }
}

impl<A, C> PaintDisplayWithCurrentTarget<A, C, MixedPaint<C>> for MixedPaintDisplayDialog<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        paint: &MixedPaint<C>,
        current_target: Option<&Colour>,
        caller: &Rc<W>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> MixedPaintDisplayDialog<A, C> {
        let dialog = new_display_dialog(&paint.name(), caller, &[]);
        dialog.set_size_from_recollections("mixed_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(Some(paint.name().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(Some(paint.notes().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        let current_target_label = gtk::Label::new(None);
        current_target_label.set_widget_colour(&paint.colour());
        vbox.pack_start(&current_target_label.clone(), true, true, 0);
        //
        let cads = A::create();
        cads.set_colour(Some(&paint.colour()));
        if let Some(matched_colour) = paint.matched_colour() {
            let matched_colour_label = gtk::Label::new(Some("Matched Colour"));
            matched_colour_label.set_widget_colour(&matched_colour.clone());
            vbox.pack_start(&matched_colour_label.clone(), true, true, 0);
            cads.set_target_colour(Some(&matched_colour.clone()));
        }
        //
        content_area.pack_start(&vbox, false, true, 0);
        content_area.pack_start(&cads.pwo(), true, true, 1);
        let characteristics_display = paint.characteristics().gui_display_widget();
        content_area.pack_start(&characteristics_display, false, false, 0);
        let components_view =
            PaintComponentListView::<A, C>::create(&paint.components(), current_target);
        content_area.pack_start(&components_view.pwo(), true, true, 0);
        content_area.show_all();
        for (response_id, spec) in button_specs.iter().enumerate() {
            let button = dialog.add_button(
                spec.label.as_str(),
                gtk::ResponseType::Other(response_id as u16),
            );
            button.set_tooltip_text(Some(spec.tooltip_text.as_str()));
        }
        dialog.connect_response(move |_, r_id| {
            if let gtk::ResponseType::Other(r_id) = r_id {
                if (r_id as usize) < button_specs.len() {
                    (button_specs[r_id as usize].callback)()
                }
            }
        });
        let spd_dialog = Rc::new(MixedPaintDisplayDialogCore {
            dialog: dialog,
            paint: paint.clone(),
            current_target_label: current_target_label,
            cads: cads,
            components_view: components_view,
            id_no: get_id_for_dialog(),
            destroyed_callbacks: DestroyedCallbacks::create(),
        });
        spd_dialog.set_current_target(current_target);
        let spd_dialog_c = spd_dialog.clone();
        spd_dialog
            .dialog
            .connect_destroy(move |_| spd_dialog_c.inform_destroyed());

        spd_dialog
    }

    fn paint(&self) -> MixedPaint<C> {
        self.paint.clone()
    }

    fn set_current_target(&self, new_current_target: Option<&Colour>) {
        if let Some(ref colour) = new_current_target {
            self.current_target_label.set_label("Current Target");
            self.current_target_label.set_widget_colour(&colour);
            self.cads.set_target_colour(Some(&colour));
            self.components_view.set_target_colour(Some(&colour));
        } else {
            self.current_target_label.set_label("");
            self.current_target_label
                .set_widget_colour(&self.paint.colour());
            if let Some(matched_colour) = self.paint.matched_colour() {
                self.cads.set_target_colour(Some(&matched_colour));
            } else {
                self.cads.set_target_colour(None);
            }
            self.components_view.set_target_colour(None);
        };
    }
}

pub const PC_NAME: i32 = SP_NAME;
pub const PC_NOTES: i32 = SP_NOTES;
pub const PC_CHROMA: i32 = SP_CHROMA;
pub const PC_GREYNESS: i32 = SP_GREYNESS;
pub const PC_VALUE: i32 = SP_VALUE;
pub const PC_WARMTH: i32 = SP_WARMTH;
pub const PC_RGB: i32 = SP_RGB;
pub const PC_RGB_FG: i32 = SP_RGB_FG;
pub const PC_MONO_RGB: i32 = SP_MONO_RGB;
pub const PC_MONO_RGB_FG: i32 = SP_MONO_RGB_FG;
pub const PC_WARMTH_RGB: i32 = SP_WARMTH_RGB;
pub const PC_WARMTH_RGB_FG: i32 = SP_WARMTH_RGB_FG;
pub const PC_HUE_RGB: i32 = SP_HUE_RGB;
pub const PC_HUE_ANGLE: i32 = SP_HUE_ANGLE;
pub const PC_PAINT_INDEX: i32 = 14;
pub const PC_PARTS: i32 = 15;
pub const PC_CHARS_0: i32 = 16;
pub const PC_CHARS_1: i32 = 17;
pub const PC_CHARS_2: i32 = 18;
pub const PC_CHARS_3: i32 = 19;

lazy_static! {
    pub static ref PAINT_COMPONENTS_ROW_SPEC: [glib::Type; 20] =
        [
            glib::Type::String,          // 0 Name
            glib::Type::String,          // 1 Notes
            glib::Type::String,          // 2 Chroma
            glib::Type::String,          // 3 Greyness
            glib::Type::String,          // 4 Value
            glib::Type::String,          // 5 Warmth
            gdk::RGBA::static_type(),   // 6 RGB
            gdk::RGBA::static_type(),   // 7 FG for RGB
            gdk::RGBA::static_type(),   // 8 Monochrome RGB
            gdk::RGBA::static_type(),   // 9 FG for Monochrome RGB
            gdk::RGBA::static_type(),   // 10 Warmth RGB
            gdk::RGBA::static_type(),   // 11 FG for Warmth RGB
            gdk::RGBA::static_type(),   // 12 Hue Colour
            f64::static_type(),         // 13 Hue angle (radians)
            u32::static_type(),         // 14 Paint Index
            glib::Type::String,          // 15 Parts
            glib::Type::String,          // 16 Characteristic #1
            glib::Type::String,          // 17 Characteristic #2
            glib::Type::String,          // 18 Characteristic #3
            glib::Type::String,          // 19 Characteristic #4
        ];
}

impl<C> PaintComponent<C>
where
    C: CharacteristicsInterface + 'static,
{
    fn tv_rows(&self, index: u32) -> Vec<glib::Value> {
        let rgba: gdk::RGBA = self.paint.rgb().into_gdk_rgba();
        let frgba: gdk::RGBA = self.paint.rgb().best_foreground_rgb().into_gdk_rgba();
        let mrgba: gdk::RGBA = self.paint.monochrome_rgb().into_gdk_rgba();
        let mfrgba: gdk::RGBA = self
            .paint
            .monochrome_rgb()
            .best_foreground_rgb()
            .into_gdk_rgba();
        let wrgba: gdk::RGBA = self.paint.warmth_rgb().into_gdk_rgba();
        let wfrgba: gdk::RGBA = self
            .paint
            .warmth_rgb()
            .best_foreground_rgb()
            .into_gdk_rgba();
        let hrgba: gdk::RGBA = self.paint.max_chroma_rgb().into_gdk_rgba();
        let hue_radians = if let Some(hue) = self.paint.hue() {
            hue.angle().radians()
        } else {
            0.0
        };
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
            hue_radians.to_value(),
            index.to_value(),
            format!("{:3}", self.parts).to_value(),
        ];
        for row in self.paint.characteristics().tv_rows().iter() {
            rows.push(row.clone());
        }
        rows
    }
}

pub struct PaintComponentListViewCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    popup_menu: WrappedMenu,
    components: Rc<Vec<PaintComponent<C>>>,
    chosen_paint: RefCell<Option<Paint<C>>>,
    current_target: RefCell<Option<Colour>>,
    series_paint_dialogs: RefCell<HashMap<u32, SeriesPaintDisplayDialog<A, C>>>,
    mixed_paint_dialogs: RefCell<HashMap<u32, MixedPaintDisplayDialog<A, C>>>,
    spec: PhantomData<A>,
}

impl<A, C> PaintComponentListViewCore<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn get_paint_at(&self, posn: (f64, f64)) -> Option<Paint<C>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let index: u32 = self
                        .list_store
                        .get_value(&iter, 14)
                        .get()
                        .unwrap()
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    if index as usize >= self.components.len() {
                        panic!(
                            "File: {:?} Line: {:?} outside array bounds",
                            file!(),
                            line!()
                        )
                    };
                    let paint = self.components[index as usize].paint.clone();
                    return Some(paint);
                }
            }
        }
        None
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        match ocolour {
            Some(colour) => {
                for dialog in self.series_paint_dialogs.borrow().values() {
                    dialog.set_current_target(Some(colour));
                }
                for dialog in self.mixed_paint_dialogs.borrow().values() {
                    dialog.set_current_target(Some(colour));
                }
                *self.current_target.borrow_mut() = Some(colour.clone())
            }
            None => {
                for dialog in self.series_paint_dialogs.borrow().values() {
                    dialog.set_current_target(None);
                }
                for dialog in self.mixed_paint_dialogs.borrow().values() {
                    dialog.set_current_target(None);
                }
                *self.current_target.borrow_mut() = None
            }
        }
    }
}

impl_widget_wrapper!(scrolled_window: gtk::ScrolledWindow, PaintComponentListViewCore<A, C>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
);

pub type PaintComponentListView<A, C> = Rc<PaintComponentListViewCore<A, C>>;

pub trait PaintComponentListViewInterface<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(
        components: &Rc<Vec<PaintComponent<C>>>,
        current_target: Option<&Colour>,
    ) -> PaintComponentListView<A, C>;
}

impl<A, C> PaintComponentListViewInterface<A, C> for PaintComponentListView<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create(
        components: &Rc<Vec<PaintComponent<C>>>,
        current_target: Option<&Colour>,
    ) -> PaintComponentListView<A, C> {
        let len = PC_CHARS_0 as usize + C::tv_row_len();
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

        let adj: Option<&gtk::Adjustment> = None;
        let pclv = Rc::new(PaintComponentListViewCore::<A, C> {
            scrolled_window: gtk::ScrolledWindow::new(adj, adj),
            list_store: list_store,
            popup_menu: WrappedMenu::new(&vec![]),
            components: components.clone(),
            view: view,
            chosen_paint: RefCell::new(None),
            current_target: RefCell::new(my_current_target),
            series_paint_dialogs: RefCell::new(HashMap::new()),
            mixed_paint_dialogs: RefCell::new(HashMap::new()),
            spec: PhantomData,
        });

        pclv.view.append_column(&simple_text_column(
            "Parts", PC_PARTS, PC_PARTS, -1, -1, -1, true,
        ));
        pclv.view.append_column(&simple_text_column(
            "Name", PC_NAME, PC_NAME, PC_RGB, PC_RGB_FG, -1, true,
        ));
        pclv.view.append_column(&simple_text_column(
            "Notes", PC_NOTES, PC_NOTES, PC_RGB, PC_RGB_FG, -1, true,
        ));
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
        pclv.popup_menu
            .append_item(
                "info",
                "Paint Information",
                "Display this paint's information",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *pclv_c.chosen_paint.borrow() {
                    let current_target = pclv_c.current_target.borrow().clone();
                    let target = if let Some(ref colour) = current_target {
                        Some(colour)
                    } else {
                        None
                    };
                    match paint {
                        &Paint::Mixed(ref mixed_paint) => {
                            let dialog = MixedPaintDisplayDialog::<A, C>::create(
                                mixed_paint,
                                target,
                                &pclv_c,
                                vec![],
                            );
                            let pclv_c_c = pclv_c.clone();
                            dialog.connect_destroyed(move |id| {
                                pclv_c_c.mixed_paint_dialogs.borrow_mut().remove(&id);
                            });
                            pclv_c
                                .mixed_paint_dialogs
                                .borrow_mut()
                                .insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        }
                        &Paint::Series(ref series_paint) => {
                            let dialog = SeriesPaintDisplayDialog::<A, C>::create(
                                series_paint,
                                target,
                                &pclv_c,
                                vec![],
                            );
                            let pclv_c_c = pclv_c.clone();
                            dialog.connect_destroyed(move |id| {
                                pclv_c_c.series_paint_dialogs.borrow_mut().remove(&id);
                            });
                            pclv_c
                                .series_paint_dialogs
                                .borrow_mut()
                                .insert(dialog.id_no(), dialog.clone());
                            dialog.show();
                        }
                    }
                }
            });

        let pclv_c = pclv.clone();
        pclv.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress {
                if event.get_button() == 3 {
                    let o_paint = pclv_c.get_paint_at(event.get_position());
                    pclv_c
                        .popup_menu
                        .set_sensitivities(o_paint.is_some(), &["info"]);
                    *pclv_c.chosen_paint.borrow_mut() = o_paint;
                    // TODO: needs v3_22: pclv_c.menu.popup_at_pointer(None);
                    pclv_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                }
            }
            Inhibit(false)
        });

        pclv
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
