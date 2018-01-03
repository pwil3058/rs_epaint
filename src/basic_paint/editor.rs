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
use std::rc::Rc;

use pw_gix::gtkx::paned::RememberPosition;
pub use pw_gix::wrapper::WidgetWrapper;

use colln_paint::*;
use struct_traits::SimpleCreation;

use super::*;
use super::factory::*;
use super::entry::*;

pub struct BasicPaintEditorCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    h_paned: gtk::Paned,
    basic_paint_factory: BasicPaintFactoryDisplay<A, C>,
    paint_spec_entry: BasicPaintSpecEntry<A, C>,
    cid_entry: CollnIdEntry<CID>,
    edited_paint: RefCell<Option<BasicPaint<C>>>,
    add_paint_btn: gtk::Button,
    accept_changes_btn: gtk::Button,
    reset_btn: gtk::Button,
}

impl<A, C, CID> BasicPaintEditorCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn update_button_sensitivities(&self) {
        match self.paint_spec_entry.get_status() {
            EntryStatus::EditingNoChange => {
                self.basic_paint_factory.set_initiate_edit_ok(true);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            },
            EntryStatus::EditingReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(true);
            },
            EntryStatus::EditingNotReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            },
            EntryStatus::CreatingReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(true);
                self.accept_changes_btn.set_sensitive(false);
            },
            EntryStatus::CreatingNotReadyNamed => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            },
            EntryStatus::CreatingNotReadyUnnamed => {
                self.basic_paint_factory.set_initiate_edit_ok(true);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            },
        }
    }

    fn ok_to_reset_entry(&self) -> bool {
        match self.paint_spec_entry.get_status() {
            EntryStatus::EditingNoChange => {
                true
            },
            EntryStatus::EditingReady | EntryStatus::EditingNotReady => {
                if let Some(ref edited_paint) = *self.edited_paint.borrow() {
                    let expln = format!("Unsaved changes to \"{}\" will be lost", edited_paint.name());
                    self.ask_confirm_action(&"Confirm reset?", Some(&expln))
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            },
            EntryStatus::CreatingReady| EntryStatus::CreatingNotReadyNamed => {
                self.ask_confirm_action(&"Confirm reset?", Some(&"Unsaved changes to new will be lost"))
            },
            EntryStatus::CreatingNotReadyUnnamed => {
                true
            },
        }
    }

    fn add_paint(&self, basic_paint_spec: &BasicPaintSpec<C>) {
        if let Ok(paint) = self.basic_paint_factory.add_paint(basic_paint_spec) {
            self.set_edited_paint(Some(&paint));
        } else {
            let expln = format!("Paint with the name \"{}\" already exists in the collection.", basic_paint_spec.name);
            self.warn_user("Duplicate Paint Name!", Some(&expln));
        }
    }

    fn accept_changes(&self, basic_paint_spec: &BasicPaintSpec<C>) {
        let o_edited_paint = self.edited_paint.borrow().clone();
        if let Some(ref old_paint) = o_edited_paint {
            if let Ok(paint) = self.basic_paint_factory.replace_paint(old_paint, basic_paint_spec) {
                self.set_edited_paint(Some(&paint));
            } else {
                let expln = format!("Paint with the name \"{}\" already exists in the collection.", basic_paint_spec.name);
                self.warn_user("Duplicate Paint Name!", Some(&expln));
            }
        } else {
            panic!("File: {} Line: {}", file!(), line!())
        }
    }

    fn set_edited_paint(&self, o_paint: Option<&BasicPaint<C>>) {
        if let Some(paint) = o_paint {
            // TODO: check for unsaved changes before setting edited spec
            *self.edited_paint.borrow_mut() = Some(paint.clone());
            self.paint_spec_entry.set_edited_spec(Some(paint.get_spec()))
        } else {
            *self.edited_paint.borrow_mut() = None;
            self.paint_spec_entry.set_edited_spec(None)
        };
        self.update_button_sensitivities();
    }
}

impl<A, C, CID> WidgetWrapper<gtk::Paned> for BasicPaintEditorCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn pwo(&self) -> gtk::Paned {
        self.h_paned.clone()
    }
}

pub type BasicPaintEditor<A, C, CID> = Rc<BasicPaintEditorCore<A, C, CID>>;

impl<A, C, CID> SimpleCreation for BasicPaintEditor<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn create() -> BasicPaintEditor<A, C, CID> {
        let add_paint_btn = gtk::Button::new_with_label("Add");
        add_paint_btn.set_tooltip_text("Add the paint defined by this specification to the collection");
        let accept_changes_btn = gtk::Button::new_with_label("Accept");
        accept_changes_btn.set_tooltip_text("Accept the changes to the paint being edited");
        let reset_btn = gtk::Button::new_with_label("Reset");
        reset_btn.set_tooltip_text("Reset in preparation for defining a new paint");
        let extra_buttons = vec![add_paint_btn.clone(), accept_changes_btn.clone(), reset_btn.clone()];

        let bpe = Rc::new(
            BasicPaintEditorCore::<A, C, CID> {
                h_paned: gtk::Paned::new(gtk::Orientation::Horizontal),
                basic_paint_factory: BasicPaintFactoryDisplay::<A, C>::create(),
                paint_spec_entry: BasicPaintSpecEntry::<A, C>::create(&extra_buttons),
                cid_entry: CollnIdEntry::<CID>::create(),
                edited_paint: RefCell::new(None),
                add_paint_btn: add_paint_btn,
                accept_changes_btn: accept_changes_btn,
                reset_btn: reset_btn,
            }
        );
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        vbox.pack_start(&bpe.cid_entry.pwo(), false, false, 0);
        vbox.pack_start(&bpe.basic_paint_factory.pwo(), true, true, 0);
        bpe.h_paned.add1(&vbox);
        bpe.h_paned.add2(&bpe.paint_spec_entry.pwo());
        bpe.h_paned.set_position_from_recollections("basic_paint_editor", 400);

        let bpe_c = bpe.clone();
        bpe.basic_paint_factory.connect_paint_removed(
            move |removed_paint| {
                let o_edited_paint = bpe_c.edited_paint.borrow().clone();
                if let Some(ref edited_paint) = o_edited_paint {
                    if *edited_paint == *removed_paint {
                        bpe_c.set_edited_paint(None)
                    }
                }
            }
        );

        let bpe_c = bpe.clone();
        bpe.basic_paint_factory.connect_edit_paint(
            move |paint| {
                bpe_c.set_edited_paint(Some(paint))
            }
        );

        let bpe_c = bpe.clone();
        bpe.paint_spec_entry.connect_status_changed(
            move |_| {
                bpe_c.update_button_sensitivities()
            }
        );

        let bpe_c = bpe.clone();
        bpe.add_paint_btn.connect_clicked(
            move |_| {
                if let Some(basic_paint_spec) = bpe_c.paint_spec_entry.get_basic_paint_spec() {
                    bpe_c.add_paint(&basic_paint_spec);
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            }
        );

        let bpe_c = bpe.clone();
        bpe.accept_changes_btn.connect_clicked(
            move |_| {
                if let Some(basic_paint_spec) = bpe_c.paint_spec_entry.get_basic_paint_spec() {
                    bpe_c.accept_changes(&basic_paint_spec);
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            }
        );

        let bpe_c = bpe.clone();
        bpe.reset_btn.connect_clicked(
            move |_| {
                if bpe_c.ok_to_reset_entry(){
                    bpe_c.set_edited_paint(None)
                }
            }
        );

        bpe.update_button_sensitivities();

        bpe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
