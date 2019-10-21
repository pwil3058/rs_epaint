// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use gdk;
use glib::signal::SignalHandlerId;
use gtk;
use gtk::prelude::*;

use pw_gix::cairox::*;
use pw_gix::gtkx::list_store::*;
use pw_gix::gtkx::menu::*;
use pw_gix::gtkx::paned::*;
use pw_gix::gtkx::tree_view_column::*;

use crate::basic_paint::*;
use crate::graticule::*;
use crate::shape::*;

use super::display::*;
use super::*;

// COLLECTION
pub struct CollnPaintCollnCore<C, CID>
where
    C: CharacteristicsInterface,
    CID: CollnIdInterface,
{
    colln_id: Rc<CID>,
    paints: Rc<Vec<CollnPaint<C, CID>>>,
}

impl<C, CID> CollnPaintCollnCore<C, CID>
where
    C: CharacteristicsInterface,
    CID: CollnIdInterface,
{
    fn find_name(&self, name: &str) -> Result<usize, usize> {
        self.paints
            .binary_search_by_key(&name.to_string(), |paint| paint.name())
    }

    pub fn colln_id(&self) -> Rc<CID> {
        self.colln_id.clone()
    }

    pub fn len(&self) -> usize {
        self.paints.len()
    }

    pub fn get_paint(&self, name: &str) -> Option<CollnPaint<C, CID>> {
        match self.find_name(name) {
            Ok(index) => Some(self.paints[index].clone()),
            Err(_) => None,
        }
    }

    pub fn get_paints(&self) -> Rc<Vec<CollnPaint<C, CID>>> {
        self.paints.clone()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.find_name(name).is_ok()
    }
}

pub type CollnPaintColln<C, CID> = Rc<CollnPaintCollnCore<C, CID>>;

pub trait CollnPaintCollnInterface<C, CID>
where
    C: CharacteristicsInterface,
    CID: CollnIdInterface,
{
    fn from_spec(colln_spec: &PaintCollnSpec<C, CID>) -> CollnPaintColln<C, CID>;
}

impl<C, CID> CollnPaintCollnInterface<C, CID> for CollnPaintColln<C, CID>
where
    C: CharacteristicsInterface,
    CID: CollnIdInterface,
{
    fn from_spec(colln_spec: &PaintCollnSpec<C, CID>) -> CollnPaintColln<C, CID> {
        let colln_id = colln_spec.colln_id.clone();
        let mut paints: Vec<CollnPaint<C, CID>> = Vec::new();
        for paint_spec in colln_spec.paint_specs.iter() {
            // Assume that the spec list is ordered and names are unique
            let basic_paint = BasicPaint::<C>::from_spec(paint_spec);
            let colln_paint = CollnPaint::<C, CID>::create(&basic_paint, &colln_id);
            paints.push(colln_paint);
        }
        Rc::new(CollnPaintCollnCore::<C, CID> {
            colln_id: colln_id,
            paints: Rc::new(paints),
        })
    }
}

pub struct CollnPaintCollnViewCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    scrolled_window: gtk::ScrolledWindow,
    list_store: gtk::ListStore,
    view: gtk::TreeView,
    colln: CollnPaintColln<C, CID>,
    phantom_data: PhantomData<A>,
}

impl<A, C, CID> CollnPaintCollnViewCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    pub fn get_paint_at(&self, posn: (f64, f64)) -> Option<CollnPaint<C, CID>> {
        let x = posn.0 as i32;
        let y = posn.1 as i32;
        if let Some(location) = self.view.get_path_at_pos(x, y) {
            if let Some(path) = location.0 {
                if let Some(iter) = self.list_store.get_iter(&path) {
                    let name: String = self
                        .list_store
                        .get_value(&iter, 0)
                        .get()
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    let paint = self
                        .colln
                        .get_paint(&name)
                        .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                    return Some(paint);
                }
            }
        };
        None
    }

    pub fn colln_id(&self) -> Rc<CID> {
        self.colln.colln_id()
    }

    pub fn len(&self) -> usize {
        self.colln.len()
    }

    pub fn get_paint(&self, name: &str) -> Option<CollnPaint<C, CID>> {
        self.colln.get_paint(name)
    }

    pub fn get_paints(&self) -> Rc<Vec<CollnPaint<C, CID>>> {
        self.colln.get_paints()
    }

    pub fn has_paint_named(&self, name: &str) -> bool {
        self.colln.has_paint_named(name)
    }

    pub fn connect_button_press_event<
        F: Fn(&gtk::TreeView, &gdk::EventButton) -> Inhibit + 'static,
    >(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.view.connect_button_press_event(f)
    }
}

impl_widget_wrapper!(scrolled_window: gtk::ScrolledWindow, CollnPaintCollnViewCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface,
);

pub type CollnPaintCollnView<A, C, CID> = Rc<CollnPaintCollnViewCore<A, C, CID>>;

pub trait CollnPaintCollnViewInterface<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    fn create(colln: &CollnPaintColln<C, CID>) -> CollnPaintCollnView<A, C, CID>;
}

impl<A, C, CID> CollnPaintCollnViewInterface<A, C, CID> for CollnPaintCollnView<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    fn create(colln: &CollnPaintColln<C, CID>) -> CollnPaintCollnView<A, C, CID> {
        let len = CollnPaint::<C, CID>::tv_row_len();
        let list_store = gtk::ListStore::new(&STANDARD_PAINT_ROW_SPEC[0..len]);
        for paint in colln.get_paints().iter() {
            list_store.append_row(&paint.tv_rows());
        }
        let view = gtk::TreeView::new_with_model(&list_store.clone());
        view.set_headers_visible(true);
        view.get_selection().set_mode(gtk::SelectionMode::None);

        let adj: Option<&gtk::Adjustment> = None;
        let mspl = Rc::new(CollnPaintCollnViewCore::<A, C, CID> {
            scrolled_window: gtk::ScrolledWindow::new(adj, adj),
            list_store: list_store,
            colln: colln.clone(),
            view: view,
            phantom_data: PhantomData,
        });

        mspl.view.append_column(&simple_text_column(
            "Name", SP_NAME, SP_NAME, SP_RGB, SP_RGB_FG, -1, true,
        ));
        mspl.view.append_column(&simple_text_column(
            "Notes", SP_NOTES, SP_NOTES, SP_RGB, SP_RGB_FG, -1, true,
        ));
        for col in A::tv_columns() {
            mspl.view.append_column(&col);
        }
        for col in C::tv_columns(SP_CHARS_0) {
            mspl.view.append_column(&col);
        }

        mspl.view.show_all();

        mspl.scrolled_window.add(&mspl.view.clone());
        mspl.scrolled_window.show_all();

        mspl
    }
}

// SHAPE

pub struct CollnPaintShape<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    paint: CollnPaint<C, CID>,
    xy: Point,
}

impl<C, CID> ColourShapeInterface for CollnPaintShape<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface,
{
    fn xy(&self) -> Point {
        self.xy
    }

    fn fill_rgb(&self) -> RGB {
        self.paint.rgb()
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Square
    }
}

impl<C, CID> ColouredItemShapeInterface<CollnPaint<C, CID>> for CollnPaintShape<C, CID>
where
    C: CharacteristicsInterface,
    CID: CollnIdInterface,
{
    fn new(paint: &CollnPaint<C, CID>, attr: ScalarAttribute) -> CollnPaintShape<C, CID> {
        CollnPaintShape::<C, CID> {
            paint: paint.clone(),
            xy: Self::colour_xy(paint.colour(), attr),
        }
    }

    fn coloured_item(&self) -> CollnPaint<C, CID> {
        self.paint.clone()
    }
}

pub type CollnPaintShapeList<C, CID> =
    ColouredItemSpapeList<CollnPaint<C, CID>, CollnPaintShape<C, CID>>;

// WHEEL
pub struct CollnPaintHueAttrWheelCore<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    paints: CollnPaintShapeList<C, CID>,
    graticule: Graticule,
}

impl_widget_wrapper!(graticule.drawing_area() -> gtk::DrawingArea, CollnPaintHueAttrWheelCore<C, CID>
    where   C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
);

pub type CollnPaintHueAttrWheel<C, CID> = Rc<CollnPaintHueAttrWheelCore<C, CID>>;

pub trait CollnPaintHueAttrWheelInterface<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(
        attr: ScalarAttribute,
        paints: Rc<Vec<CollnPaint<C, CID>>>,
    ) -> CollnPaintHueAttrWheel<C, CID>;
}

impl<C, CID> CollnPaintHueAttrWheelInterface<C, CID> for CollnPaintHueAttrWheel<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(
        attr: ScalarAttribute,
        paints: Rc<Vec<CollnPaint<C, CID>>>,
    ) -> CollnPaintHueAttrWheel<C, CID> {
        let wheel = Rc::new(CollnPaintHueAttrWheelCore::<C, CID> {
            paints: CollnPaintShapeList::<C, CID>::new(attr),
            graticule: Graticule::create(attr),
        });
        for paint in paints.iter() {
            wheel.add_paint(paint)
        }

        let wheel_c = wheel.clone();
        wheel
            .graticule
            .drawing_area()
            .connect_query_tooltip(move |_, x, y, _, tooltip| {
                // TODO: find out why tooltip.set_tip_area() nobbles tooltips
                //let rectangle = gtk::Rectangle{x: x, y: y, width: 10, height: -10};
                //println!("Rectangle: {:?}", rectangle);
                //tooltip.set_tip_area(&rectangle);
                match wheel_c.get_paint_at((x as f64, y as f64)) {
                    Some(paint) => {
                        tooltip.set_text(Some(paint.tooltip_text().as_str()));
                        true
                    }
                    None => false,
                }
            });
        let wheel_c = wheel.clone();
        wheel
            .graticule
            .connect_draw(move |graticule, cairo_context| {
                cairo_context.set_line_width(2.0);
                wheel_c.paints.draw(graticule, cairo_context);
            });
        wheel
    }
}

impl<C, CID> CollnPaintHueAttrWheelCore<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn add_paint(&self, paint: &CollnPaint<C, CID>) {
        self.paints.add_coloured_item(paint);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_paint_at(&self, posn: (f64, f64)) -> Option<CollnPaint<C, CID>> {
        let point = self.graticule.reverse_transform(Point::from(posn));
        let opr = self.paints.get_coloured_item_at(point);
        if let Some((paint, _)) = opr {
            Some(paint)
        } else {
            None
        }
    }

    pub fn connect_button_press_event<
        F: Fn(&gtk::DrawingArea, &gdk::EventButton) -> Inhibit + 'static,
    >(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.graticule.connect_button_press_event(f)
    }
}

// WIDGET
pub struct CollnPaintCollnWidgetCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    vbox: gtk::Box,
    hue_attr_wheels: Vec<CollnPaintHueAttrWheel<C, CID>>,
    paint_colln_view: CollnPaintCollnView<A, C, CID>,
    popup_menu: WrappedMenu,
    paint_dialogs: RefCell<HashMap<u32, CollnPaintDisplayDialog<A, C, CID>>>,
    initiate_select_ok: Cell<bool>,
    chosen_paint: RefCell<Option<CollnPaint<C, CID>>>,
    current_target: RefCell<Option<Colour>>,
    paint_selected_callbacks: RefCell<Vec<Box<dyn Fn(&CollnPaint<C, CID>)>>>,
}

impl_widget_wrapper!(vbox: gtk::Box, CollnPaintCollnWidgetCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
);

pub type CollnPaintCollnWidget<A, C, CID> = Rc<CollnPaintCollnWidgetCore<A, C, CID>>;

pub trait CollnPaintCollnWidgetInterface<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(colln_spec: &PaintCollnSpec<C, CID>) -> CollnPaintCollnWidget<A, C, CID>;
}

impl<A, C, CID> CollnPaintCollnWidgetCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    pub fn colln_id(&self) -> Rc<CID> {
        self.paint_colln_view.colln_id()
    }

    fn inform_paint_selected(&self, paint: &CollnPaint<C, CID>) {
        for callback in self.paint_selected_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    pub fn set_initiate_select_ok(&self, value: bool) {
        self.initiate_select_ok.set(value);
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.set_response_sensitive(gtk::ResponseType::Other(0), value);
        }
    }

    pub fn connect_paint_selected<F: 'static + Fn(&CollnPaint<C, CID>)>(&self, callback: F) {
        self.paint_selected_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        for wheel in self.hue_attr_wheels.iter() {
            wheel.set_target_colour(o_colour);
        }
        for dialog in self.paint_dialogs.borrow().values() {
            dialog.set_current_target(o_colour);
        }
        if let Some(colour) = o_colour {
            *self.current_target.borrow_mut() = Some(colour.clone())
        } else {
            *self.current_target.borrow_mut() = None
        }
    }
}

impl<A, C, CID> CollnPaintCollnWidgetInterface<A, C, CID> for CollnPaintCollnWidget<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(colln_spec: &PaintCollnSpec<C, CID>) -> CollnPaintCollnWidget<A, C, CID> {
        let paint_colln = CollnPaintColln::<C, CID>::from_spec(colln_spec);
        let mut view_attr_wheels: Vec<CollnPaintHueAttrWheel<C, CID>> = Vec::new();
        for attr in A::scalar_attributes().iter() {
            view_attr_wheels.push(CollnPaintHueAttrWheel::<C, CID>::create(
                *attr,
                paint_colln.get_paints(),
            ));
        }
        let cpcw = Rc::new(CollnPaintCollnWidgetCore::<A, C, CID> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            hue_attr_wheels: view_attr_wheels,
            paint_colln_view: CollnPaintCollnView::<A, C, CID>::create(&paint_colln),
            paint_dialogs: RefCell::new(HashMap::new()),
            popup_menu: WrappedMenu::new(&vec![]),
            initiate_select_ok: Cell::new(false),
            chosen_paint: RefCell::new(None),
            current_target: RefCell::new(None),
            paint_selected_callbacks: RefCell::new(Vec::new()),
        });
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let colln_name = format!(
            "{} {}",
            CID::colln_name_label(),
            colln_spec.colln_id.colln_name()
        );
        hbox.pack_start(&gtk::Label::new(Some(colln_name.as_str())), true, true, 0);
        let colln_owner = format!(
            "{} {}",
            CID::colln_owner_label(),
            colln_spec.colln_id.colln_owner()
        );
        hbox.pack_start(&gtk::Label::new(Some(colln_owner.as_str())), true, true, 0);

        let notebook = gtk::Notebook::new();
        for wheel in cpcw.hue_attr_wheels.iter() {
            let label_text = format!("Hue/{} Wheel", wheel.attr().to_string());
            let label = gtk::Label::new(Some(label_text.as_str()));
            notebook.append_page(&wheel.pwo(), Some(&label));
        }
        notebook.set_scrollable(true);
        notebook.popup_enable();
        let hpaned = gtk::Paned::new(gtk::Orientation::Horizontal);
        hpaned.pack1(&notebook, true, true);
        hpaned.pack2(&cpcw.paint_colln_view.pwo(), true, true);
        hpaned.set_position_from_recollections("colln_paint_colln_widget", 200);
        cpcw.vbox.pack_start(&hpaned, true, true, 0);

        let cpcw_c = cpcw.clone();
        cpcw.popup_menu
            .append_item(
                "info",
                "Paint Information",
                "Display this paint's information",
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *cpcw_c.chosen_paint.borrow() {
                    let cpcw_c_c = cpcw_c.clone();
                    let paint_c = paint.clone();
                    let select_btn_spec = PaintDisplayButtonSpec {
                        label: CID::paint_select_label().to_string(),
                        tooltip_text: CID::paint_select_tooltip_text().to_string(),
                        callback: Box::new(move || cpcw_c_c.inform_paint_selected(&paint_c)),
                    };
                    let dialog = if CID::display_current_target() {
                        let target_colour = cpcw_c.current_target.borrow().clone();
                        let target = if let Some(ref colour) = target_colour {
                            Some(colour)
                        } else {
                            None
                        };
                        CollnPaintDisplayDialog::<A, C, CID>::create(
                            &paint,
                            target,
                            &cpcw_c,
                            vec![select_btn_spec],
                        )
                    } else {
                        CollnPaintDisplayDialog::<A, C, CID>::create(
                            &paint,
                            None,
                            &cpcw_c,
                            vec![select_btn_spec],
                        )
                    };
                    dialog.set_response_sensitive(
                        gtk::ResponseType::Other(0),
                        cpcw_c.initiate_select_ok.get(),
                    );
                    let cpcw_c_c = cpcw_c.clone();
                    dialog.connect_destroyed(move |id| {
                        cpcw_c_c.paint_dialogs.borrow_mut().remove(&id);
                    });
                    cpcw_c
                        .paint_dialogs
                        .borrow_mut()
                        .insert(dialog.id_no(), dialog.clone());
                    dialog.show();
                }
            });

        let cpcw_c = cpcw.clone();
        cpcw.popup_menu
            .append_item(
                "select",
                &CID::paint_select_label(),
                &CID::paint_select_tooltip_text(),
            )
            .connect_activate(move |_| {
                if let Some(ref paint) = *cpcw_c.chosen_paint.borrow() {
                    cpcw_c.inform_paint_selected(paint)
                }
            });

        let cpcw_c = cpcw.clone();
        cpcw.paint_colln_view
            .connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    if let Some(paint) = cpcw_c.paint_colln_view.get_paint_at(event.get_position())
                    {
                        cpcw_c
                            .popup_menu
                            .set_sensitivities(cpcw_c.initiate_select_ok.get(), &["select"]);
                        cpcw_c.popup_menu.set_sensitivities(true, &["info"]);
                        *cpcw_c.chosen_paint.borrow_mut() = Some(paint);
                    } else {
                        cpcw_c
                            .popup_menu
                            .set_sensitivities(false, &["info", "select"]);
                        *cpcw_c.chosen_paint.borrow_mut() = None;
                    };
                    cpcw_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                };
                Inhibit(false)
            });

        for wheel in cpcw.hue_attr_wheels.iter() {
            let cpcw_c = cpcw.clone();
            let wheel_c = wheel.clone();
            wheel.connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    if let Some(paint) = wheel_c.get_paint_at(event.get_position()) {
                        cpcw_c
                            .popup_menu
                            .set_sensitivities(cpcw_c.initiate_select_ok.get(), &["select"]);
                        cpcw_c.popup_menu.set_sensitivities(true, &["info"]);
                        *cpcw_c.chosen_paint.borrow_mut() = Some(paint);
                    } else {
                        cpcw_c
                            .popup_menu
                            .set_sensitivities(false, &["info", "select"]);
                        *cpcw_c.chosen_paint.borrow_mut() = None;
                    };
                    cpcw_c.popup_menu.popup_at_event(event);
                    return Inhibit(true);
                };
                Inhibit(false)
            });
        }

        cpcw
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
