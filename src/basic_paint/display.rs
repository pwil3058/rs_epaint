// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::dialog::*;

use crate::basic_paint::*;
pub use crate::dialogue::*;

pub struct BasicPaintDisplayDialogCore<A, C>
where
    C: CharacteristicsInterface + 'static,
    A: ColourAttributesInterface + 'static,
{
    dialog: gtk::Dialog,
    paint: BasicPaint<C>,
    _cads: Rc<A>,
    id_no: u32,
    destroyed_callbacks: DestroyedCallbacks,
}

pub type BasicPaintDisplayDialog<A, C> = Rc<BasicPaintDisplayDialogCore<A, C>>;

impl<A, C> DialogWrapper for BasicPaintDisplayDialog<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn dialog(&self) -> gtk::Dialog {
        self.dialog.clone()
    }
}

impl<A, C> TrackedDialog for BasicPaintDisplayDialog<A, C>
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

impl<A, C> PaintDisplay<A, C, BasicPaint<C>> for BasicPaintDisplayDialog<A, C>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
{
    fn create<W: WidgetWrapper>(
        paint: &BasicPaint<C>,
        caller: &Rc<W>,
        button_specs: Vec<PaintDisplayButtonSpec>,
    ) -> BasicPaintDisplayDialog<A, C> {
        let dialog = new_display_dialog(&paint.name(), caller, &[]);
        dialog.set_size_from_recollections("basic_paint_display", (60, 330));
        let content_area = dialog.get_content_area();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(Some(paint.name().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        let label = gtk::Label::new(Some(paint.notes().as_str()));
        label.set_widget_colour(&paint.colour());
        vbox.pack_start(&label, false, false, 0);
        //
        content_area.pack_start(&vbox, true, true, 0);
        let cads = A::create();
        cads.set_colour(Some(&paint.colour()));
        content_area.pack_start(&cads.pwo(), true, true, 1);
        let characteristics_display = paint.characteristics().gui_display_widget();
        content_area.pack_start(&characteristics_display, false, false, 0);
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
        let bpd_dialog = Rc::new(BasicPaintDisplayDialogCore {
            dialog: dialog,
            paint: paint.clone(),
            _cads: cads,
            id_no: get_id_for_dialog(),
            destroyed_callbacks: DestroyedCallbacks::create(),
        });
        let bpd_dialog_c = bpd_dialog.clone();
        bpd_dialog
            .dialog
            .connect_destroy(move |_| bpd_dialog_c.inform_destroyed());

        bpd_dialog
    }

    fn paint(&self) -> BasicPaint<C> {
        self.paint.clone()
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
