// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use pw_gix::{
    gdk,
    glib::signal::SignalHandlerId,
    gtk::{self, prelude::*},
    wrapper::*,
};

use crate::basic_paint::*;
use crate::cairox::*;
use crate::graticule::*;
use crate::shape::*;

// BASIC PAINT
pub struct BasicPaintShape<C>
where
    C: CharacteristicsInterface + 'static,
{
    paint: BasicPaint<C>,
    xy: Point,
    phantom: PhantomData<C>,
}

impl<C> ColourShapeInterface for BasicPaintShape<C>
where
    C: CharacteristicsInterface + 'static,
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

impl<C> ColouredItemShapeInterface<BasicPaint<C>> for BasicPaintShape<C>
where
    C: CharacteristicsInterface,
{
    fn new(paint: &BasicPaint<C>, attr: ScalarAttribute) -> BasicPaintShape<C> {
        BasicPaintShape::<C> {
            paint: paint.clone(),
            xy: Self::colour_xy(paint.colour(), attr),
            phantom: PhantomData,
        }
    }

    fn coloured_item(&self) -> BasicPaint<C> {
        self.paint.clone()
    }
}

pub type BasicPaintShapeList<C> = ColouredItemSpapeList<BasicPaint<C>, BasicPaintShape<C>>;

// WHEEL
#[derive(Wrapper)]
pub struct BasicPaintHueAttrWheelCore<C>
where
    C: CharacteristicsInterface + 'static,
{
    paints: BasicPaintShapeList<C>,
    chosen_paint: RefCell<Option<BasicPaint<C>>>,
    graticule: Graticule,
}

impl<C> PackableWidgetObject for BasicPaintHueAttrWheelCore<C>
where
    C: CharacteristicsInterface + 'static,
{
    type PWT = gtk::DrawingArea;

    fn pwo(&self) -> Self::PWT {
        self.graticule.drawing_area().clone()
    }
}

pub type BasicPaintHueAttrWheel<C> = Rc<BasicPaintHueAttrWheelCore<C>>;

pub trait BasicPaintHueAttrWheelInterface<C>
where
    C: CharacteristicsInterface + 'static,
{
    fn create(attr: ScalarAttribute) -> BasicPaintHueAttrWheel<C>;
}

impl<C> BasicPaintHueAttrWheelInterface<C> for BasicPaintHueAttrWheel<C>
where
    C: CharacteristicsInterface + 'static,
{
    fn create(attr: ScalarAttribute) -> BasicPaintHueAttrWheel<C> {
        let wheel = Rc::new(BasicPaintHueAttrWheelCore::<C> {
            paints: BasicPaintShapeList::<C>::new(attr),
            graticule: Graticule::create(attr),
            chosen_paint: RefCell::new(None),
        });
        let wheel_c = wheel.clone();
        wheel
            .paints
            .connect_changed(move || wheel_c.graticule.queue_draw());

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

impl<C> BasicPaintHueAttrWheelCore<C>
where
    C: CharacteristicsInterface + 'static,
{
    pub fn clear(&self) {
        *self.chosen_paint.borrow_mut() = None;
        self.paints.clear();
    }

    pub fn add_paint(&self, paint: &BasicPaint<C>) {
        self.paints.add_coloured_item(paint);
    }

    pub fn remove_paint(&self, paint: &BasicPaint<C>) {
        self.paints.remove_coloured_item(paint);
    }

    pub fn replace_paint(&self, old_paint: &BasicPaint<C>, paint: &BasicPaint<C>) {
        self.paints.replace_coloured_item(old_paint, paint);
    }

    pub fn set_target_colour(&self, o_colour: Option<&Colour>) {
        self.graticule.set_current_target_colour(o_colour);
    }

    pub fn attr(&self) -> ScalarAttribute {
        self.graticule.attr()
    }

    pub fn get_paint_at(&self, posn: (f64, f64)) -> Option<BasicPaint<C>> {
        let point = self.graticule.reverse_transform(Point::from(posn));
        let opr = self.paints.get_coloured_item_at(point);
        if let Some((paint, _)) = opr {
            Some(paint)
        } else {
            None
        }
    }

    pub fn set_chosen_paint_from(&self, posn: (f64, f64)) -> Option<BasicPaint<C>> {
        if let Some(paint) = self.get_paint_at(posn) {
            *self.chosen_paint.borrow_mut() = Some(paint.clone());
            Some(paint)
        } else {
            *self.chosen_paint.borrow_mut() = None;
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

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
