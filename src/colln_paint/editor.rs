// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use pw_gix::gtkx::coloured::*;
use pw_gix::gtkx::paned::RememberPosition;
use pw_gix::recollections::{recall, remember};
pub use pw_gix::wrapper::WidgetWrapper;

use pw_pathux;

use crate::icons::colln_xpms;
use crate::icons::file_status_xpms::*;

pub use crate::struct_traits::SimpleCreation;

use crate::basic_paint::entry::*;
use crate::basic_paint::factory::*;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileStatus {
    NoFileNoData,
    NoFileDataReady,
    NoFileDataNotReady,
    UpToDate,
    NotUpToDateReady,
    NotUpToDateNotReady,
}

impl FileStatus {
    pub fn needs_saving(&self) -> bool {
        match *self {
            FileStatus::UpToDate | FileStatus::NoFileNoData => false,
            _ => true,
        }
    }

    pub fn is_saveable(&self) -> bool {
        match *self {
            FileStatus::UpToDate | FileStatus::NoFileDataReady | FileStatus::NotUpToDateReady => {
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
struct FileData<C, CID>
where
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    pub path: PathBuf,
    pub spec: PaintCollnSpec<C, CID>,
}

#[derive(PWO, Wrapper)]
pub struct CollnPaintEditorCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    vbox: gtk::Box,
    h_paned: gtk::Paned,
    basic_paint_factory: BasicPaintFactoryDisplay<A, C>,
    paint_spec_entry: BasicPaintSpecEntry<A, C>,
    cid_entry: CollnIdEntry<CID>,
    edited_paint: RefCell<Option<BasicPaint<C>>>,
    add_paint_btn: gtk::Button,
    accept_changes_btn: gtk::Button,
    reset_entry_btn: gtk::Button,
    // File control
    file_data: RefCell<Option<FileData<C, CID>>>,
    file_path_text: gtk::Label,
    new_colln_btn: gtk::Button,
    load_colln_btn: gtk::Button,
    save_colln_btn: gtk::Button,
    save_as_colln_btn: gtk::Button,
    file_status_btn: gtk::Button,
}

impl<A, C, CID> CollnPaintEditorCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn saved_file_path(&self) -> Option<PathBuf> {
        if let Some(ref file_data) = *self.file_data.borrow() {
            Some(file_data.path.clone())
        } else {
            None
        }
    }

    fn update_file_status_button(&self, file_status: FileStatus) {
        match file_status {
            FileStatus::NoFileNoData | FileStatus::UpToDate => {
                self.file_status_btn.set_sensitive(false);
                self.file_status_btn.set_image(Some(&up_to_date_image(24)));
                self.file_status_btn
                    .set_tooltip_text(Some("File Status: Up To Date"));
            }
            FileStatus::NoFileDataReady | FileStatus::NotUpToDateReady => {
                self.file_status_btn.set_sensitive(true);
                self.file_status_btn
                    .set_image(Some(&needs_save_ready_image(24)));
                self.file_status_btn.set_tooltip_text(Some(
                    "File Status: Needs Save (Ready)\nClick to save data to file",
                ));
            }
            FileStatus::NoFileDataNotReady | FileStatus::NotUpToDateNotReady => {
                self.file_status_btn.set_sensitive(false);
                self.file_status_btn
                    .set_image(Some(&needs_save_not_ready_image(24)));
                self.file_status_btn
                    .set_tooltip_text(Some("File Status: Needs Save (NOT Ready)"));
            }
        }
    }

    fn update_file_button_sensitivities_using(&self, file_status: FileStatus) {
        match file_status {
            FileStatus::NoFileNoData => {
                self.new_colln_btn.set_sensitive(true);
                self.load_colln_btn.set_sensitive(true);
                self.save_colln_btn.set_sensitive(false);
                self.save_as_colln_btn.set_sensitive(false);
            }
            FileStatus::NoFileDataReady => {
                self.new_colln_btn.set_sensitive(true);
                self.load_colln_btn.set_sensitive(false);
                self.save_colln_btn.set_sensitive(false);
                self.save_as_colln_btn.set_sensitive(true);
            }
            FileStatus::NoFileDataNotReady | FileStatus::NotUpToDateNotReady => {
                self.new_colln_btn.set_sensitive(true);
                self.load_colln_btn.set_sensitive(false);
                self.save_colln_btn.set_sensitive(false);
                self.save_as_colln_btn.set_sensitive(false);
            }
            FileStatus::UpToDate => {
                self.new_colln_btn.set_sensitive(true);
                self.load_colln_btn.set_sensitive(true);
                self.save_colln_btn.set_sensitive(false);
                self.save_as_colln_btn.set_sensitive(true);
            }
            FileStatus::NotUpToDateReady => {
                self.new_colln_btn.set_sensitive(true);
                self.load_colln_btn.set_sensitive(false);
                self.save_colln_btn.set_sensitive(true);
                self.save_as_colln_btn.set_sensitive(true);
            }
        };
        self.update_file_status_button(file_status);
    }

    fn update_file_button_sensitivities(&self) {
        let file_status = self.get_file_status();
        self.update_file_button_sensitivities_using(file_status);
    }

    fn update_button_sensitivities(&self) {
        let status = self.paint_spec_entry.get_status();
        match status {
            EntryStatus::EditingNoChange => {
                self.basic_paint_factory.set_initiate_edit_ok(true);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            }
            EntryStatus::EditingReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(true);
            }
            EntryStatus::EditingNotReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            }
            EntryStatus::CreatingReady => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(true);
                self.accept_changes_btn.set_sensitive(false);
            }
            EntryStatus::CreatingNotReadyNamed => {
                self.basic_paint_factory.set_initiate_edit_ok(false);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            }
            EntryStatus::CreatingNotReadyUnnamed => {
                self.basic_paint_factory.set_initiate_edit_ok(true);
                self.add_paint_btn.set_sensitive(false);
                self.accept_changes_btn.set_sensitive(false);
            }
        };
        let file_status = self.get_file_status_using(status);
        self.update_file_button_sensitivities_using(file_status);
    }

    fn get_file_status_using(&self, entry_status: EntryStatus) -> FileStatus {
        if let Some(ref file_data) = *self.file_data.borrow() {
            if entry_status.needs_saving() {
                FileStatus::NotUpToDateNotReady
            } else if let Some(cid) = self.cid_entry.get_colln_id() {
                if cid == file_data.spec.colln_id
                    && self
                        .basic_paint_factory
                        .matches_paint_specs(&file_data.spec.paint_specs)
                {
                    FileStatus::UpToDate
                } else {
                    FileStatus::NotUpToDateReady
                }
            } else {
                FileStatus::NotUpToDateNotReady
            }
        } else if self.cid_entry.get_colln_id().is_some() {
            if entry_status.needs_saving() {
                FileStatus::NoFileDataNotReady
            } else {
                FileStatus::NoFileDataReady
            }
        } else if entry_status.needs_saving() || self.basic_paint_factory.len() > 0 {
            FileStatus::NoFileDataNotReady
        } else {
            FileStatus::NoFileNoData
        }
    }

    pub fn get_file_status(&self) -> FileStatus {
        let entry_status = self.paint_spec_entry.get_status();
        self.get_file_status_using(entry_status)
    }

    fn ok_to_reset_entry(&self) -> bool {
        match self.paint_spec_entry.get_status() {
            EntryStatus::EditingNoChange => true,
            EntryStatus::EditingReady | EntryStatus::EditingNotReady => {
                if let Some(ref edited_paint) = *self.edited_paint.borrow() {
                    let expln = format!(
                        "Unsaved changes to \"{}\" will be lost",
                        edited_paint.name()
                    );
                    self.ask_confirm_action(&"Confirm reset?", Some(&expln))
                } else {
                    panic!("File: {} Line: {}", file!(), line!())
                }
            }
            EntryStatus::CreatingReady | EntryStatus::CreatingNotReadyNamed => self
                .ask_confirm_action(
                    &"Confirm reset?",
                    Some(&"Unsaved changes to new will be lost"),
                ),
            EntryStatus::CreatingNotReadyUnnamed => true,
        }
    }

    fn add_paint(&self, basic_paint_spec: &BasicPaintSpec<C>) {
        if let Ok(paint) = self.basic_paint_factory.add_paint(basic_paint_spec) {
            self.set_edited_paint(Some(&paint));
        } else {
            let expln = format!(
                "Paint with the name \"{}\" already exists in the collection.",
                basic_paint_spec.name
            );
            self.warn_user("Duplicate Paint Name!", Some(&expln));
        }
    }

    fn accept_changes(&self, basic_paint_spec: &BasicPaintSpec<C>) {
        let o_edited_paint = self.edited_paint.borrow().clone();
        if let Some(ref old_paint) = o_edited_paint {
            if let Ok(paint) = self
                .basic_paint_factory
                .replace_paint(old_paint, basic_paint_spec)
            {
                self.set_edited_paint(Some(&paint));
            } else {
                let expln = format!(
                    "Paint with the name \"{}\" already exists in the collection.",
                    basic_paint_spec.name
                );
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
            self.paint_spec_entry
                .set_edited_spec(Some(paint.get_spec()))
        } else {
            *self.edited_paint.borrow_mut() = None;
            self.paint_spec_entry.set_edited_spec(None)
        };
        self.update_button_sensitivities();
    }

    fn set_file_data(&self, o_file_data: Option<FileData<C, CID>>) {
        // TODO: update displayed file path
        *self.file_data.borrow_mut() = o_file_data;
        if let Some(ref file_path) = self.saved_file_path() {
            let path_text = pw_pathux::path_to_string(file_path);
            self.file_path_text.set_label(&path_text);
            remember(
                &CID::recollection_name_for("last_colln_edited_file"),
                &path_text,
            );
        } else {
            self.file_path_text.set_label("");
        };
        self.update_file_button_sensitivities();
    }

    fn write_to_file(&self, path: &Path) -> Result<(), PaintError<C>> {
        if let Some(colln_id) = self.cid_entry.get_colln_id() {
            let spec = PaintCollnSpec::<C, CID> {
                colln_id: colln_id,
                paint_specs: self.basic_paint_factory.get_paint_specs(),
            };
            let mut file = File::create(path)?;
            let spec_text = spec.to_string();
            match file.write(&spec_text.into_bytes()) {
                Ok(_) => {
                    let file_data = FileData::<C, CID> {
                        path: path.to_path_buf(),
                        spec: spec,
                    };
                    self.set_file_data(Some(file_data));
                    Ok(())
                }
                Err(err) => {
                    let o_current_file_data = self.file_data.borrow();
                    if let Some(ref curr_file_data) = *o_current_file_data {
                        if curr_file_data.path == path {
                            // we've trashed the file
                            self.set_file_data(None)
                        }
                    };
                    Err(err.into())
                }
            }
        } else {
            panic!("cannot save without collection id")
        }
    }

    fn save_as(&self) -> Result<(), PaintError<C>> {
        let o_last_file = recall(&CID::recollection_name_for("last_colln_edited_file"));
        let last_file = if let Some(ref text) = o_last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Save as:"), last_file, false) {
            self.write_to_file(&path)
        } else {
            Err(PaintErrorType::UserCancelled.into())
        }
    }

    fn report_save_as_failed(&self, error: &PaintError<C>) {
        match error.error_type() {
            &PaintErrorType::UserCancelled => (),
            _ => self.report_error("Failed to save file", error),
        }
    }

    pub fn ok_to_reset(&self) -> bool {
        let status = self.get_file_status();
        if status.needs_saving() {
            if status.is_saveable() {
                let buttons = [
                    ("Cancel", gtk::ResponseType::Other(0)),
                    ("Save and Continue", gtk::ResponseType::Other(1)),
                    ("Continue Discarding Changes", gtk::ResponseType::Other(2)),
                ];
                match self.ask_question("There are unsaved changes!", None, &buttons) {
                    gtk::ResponseType::Other(0) => return false,
                    gtk::ResponseType::Other(1) => {
                        if let Some(path) = self.saved_file_path() {
                            if let Err(err) = self.write_to_file(&path) {
                                self.report_error("Failed to save file", &err);
                                return false;
                            }
                        } else if let Err(err) = self.save_as() {
                            self.report_save_as_failed(&err);
                            return false;
                        };
                        return true;
                    }
                    _ => return true,
                }
            } else {
                let buttons = &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Continue Discarding Changes", gtk::ResponseType::Accept),
                ];
                return self.ask_question("There are unsaved changes!", None, buttons)
                    == gtk::ResponseType::Accept;
            }
        };
        true
    }

    pub fn reset(&self) {
        if self.ok_to_reset() {
            self.paint_spec_entry.set_edited_spec(None);
            self.cid_entry.set_colln_id(None);
            self.basic_paint_factory.clear();
            self.set_file_data(None);
        }
    }

    pub fn load_from_file(&self) {
        if !self.ok_to_reset() {
            return;
        };
        let o_last_file = recall(&CID::recollection_name_for("last_colln_edited_file"));
        let last_file = if let Some(ref text) = o_last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Load from:"), last_file, true) {
            match PaintCollnSpec::from_file(&path) {
                Ok(spec) => {
                    self.paint_spec_entry.set_edited_spec(None);
                    self.cid_entry.set_colln_id(Some(&spec.colln_id));
                    self.basic_paint_factory.clear();
                    for paint_spec in spec.paint_specs.iter() {
                        if let Err(err) = self.basic_paint_factory.add_paint(paint_spec) {
                            self.report_error("Error", &err)
                        }
                    }
                    self.set_file_data(Some(FileData { path, spec }));
                }
                Err(err) => {
                    let msg = format!("{:?}: Failed to load", path);
                    self.report_error(&msg, &err)
                }
            }
        }
    }
}

pub type CollnPaintEditor<A, C, CID> = Rc<CollnPaintEditorCore<A, C, CID>>;

impl<A, C, CID> SimpleCreation for CollnPaintEditor<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create() -> CollnPaintEditor<A, C, CID> {
        let add_paint_btn = gtk::Button::new_with_label("Add");
        add_paint_btn.set_tooltip_text(Some(
            "Add the paint defined by this specification to the collection",
        ));
        let accept_changes_btn = gtk::Button::new_with_label("Accept");
        accept_changes_btn.set_tooltip_text(Some("Accept the changes to the paint being edited"));
        let reset_entry_btn = gtk::Button::new_with_label("Reset");
        reset_entry_btn.set_tooltip_text(Some("Reset in preparation for defining a new paint"));
        let extra_buttons = vec![
            add_paint_btn.clone(),
            accept_changes_btn.clone(),
            reset_entry_btn.clone(),
        ];

        let new_colln_btn = gtk::Button::new();
        new_colln_btn.set_image(Some(&colln_xpms::colln_new_image(24)));
        new_colln_btn.set_tooltip_text(Some(
            "Clear the editor in preparation for creating a new collection",
        ));
        let load_colln_btn = gtk::Button::new();
        load_colln_btn.set_image(Some(&colln_xpms::colln_open_image(24)));
        load_colln_btn.set_tooltip_text(Some("Load a paint collection from a file for editing"));
        let save_colln_btn = gtk::Button::new();
        save_colln_btn.set_image(Some(&colln_xpms::colln_save_image(24)));
        save_colln_btn.set_tooltip_text(Some("Save the current editor content to file."));
        let save_as_colln_btn = gtk::Button::new();
        save_as_colln_btn.set_image(Some(&colln_xpms::colln_save_as_image(24)));
        save_as_colln_btn
            .set_tooltip_text(Some("Save the current editor content to a nominated file."));

        let file_status_btn = gtk::Button::new();
        file_status_btn.set_image(Some(&up_to_date_image(24)));

        let bpe = Rc::new(CollnPaintEditorCore::<A, C, CID> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 1),
            h_paned: gtk::Paned::new(gtk::Orientation::Horizontal),
            basic_paint_factory: BasicPaintFactoryDisplay::<A, C>::create(),
            paint_spec_entry: BasicPaintSpecEntry::<A, C>::create(&extra_buttons),
            cid_entry: CollnIdEntry::<CID>::create(),
            edited_paint: RefCell::new(None),
            add_paint_btn: add_paint_btn,
            accept_changes_btn: accept_changes_btn,
            reset_entry_btn: reset_entry_btn,
            file_data: RefCell::new(None),
            new_colln_btn: new_colln_btn,
            load_colln_btn: load_colln_btn,
            save_colln_btn: save_colln_btn,
            save_as_colln_btn: save_as_colln_btn,
            file_path_text: gtk::Label::new(None),
            file_status_btn: file_status_btn,
        });
        bpe.file_path_text.set_justify(gtk::Justification::Left);
        bpe.file_path_text.set_xalign(0.01);
        bpe.file_path_text.set_widget_colour_rgb(RGB::WHITE);
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        hbox.pack_start(&bpe.new_colln_btn, false, false, 0);
        hbox.pack_start(&bpe.load_colln_btn, false, false, 0);
        hbox.pack_start(&bpe.save_colln_btn, false, false, 0);
        hbox.pack_start(&bpe.save_as_colln_btn, false, false, 0);
        hbox.pack_start(&gtk::Label::new(Some("Current File:")), false, false, 0);
        hbox.pack_start(&bpe.file_path_text, true, true, 0);
        hbox.pack_start(&bpe.file_status_btn, false, false, 0);
        bpe.vbox.pack_start(&hbox, false, false, 0);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        vbox.pack_start(&bpe.cid_entry.pwo(), false, false, 0);
        vbox.pack_start(&bpe.basic_paint_factory.pwo(), true, true, 0);
        bpe.h_paned.add1(&vbox);
        bpe.h_paned.add2(&bpe.paint_spec_entry.pwo());
        bpe.h_paned
            .set_position_from_recollections("basic_paint_editor", 400);
        bpe.vbox.pack_start(&bpe.h_paned, true, true, 0);

        let bpe_c = bpe.clone();
        bpe.basic_paint_factory
            .connect_paint_removed(move |removed_paint| {
                let o_edited_paint = bpe_c.edited_paint.borrow().clone();
                if let Some(ref edited_paint) = o_edited_paint {
                    if *edited_paint == *removed_paint {
                        bpe_c.set_edited_paint(None)
                    }
                };
                bpe_c.update_file_button_sensitivities();
            });

        let bpe_c = bpe.clone();
        bpe.basic_paint_factory
            .connect_edit_paint(move |paint| bpe_c.set_edited_paint(Some(paint)));

        let bpe_c = bpe.clone();
        bpe.paint_spec_entry
            .connect_status_changed(move |_| bpe_c.update_button_sensitivities());

        let bpe_c = bpe.clone();
        bpe.add_paint_btn.connect_clicked(move |_| {
            if let Some(basic_paint_spec) = bpe_c.paint_spec_entry.get_basic_paint_spec() {
                bpe_c.add_paint(&basic_paint_spec);
            } else {
                panic!("File: {} Line: {}", file!(), line!())
            }
        });

        let bpe_c = bpe.clone();
        bpe.accept_changes_btn.connect_clicked(move |_| {
            if let Some(basic_paint_spec) = bpe_c.paint_spec_entry.get_basic_paint_spec() {
                bpe_c.accept_changes(&basic_paint_spec);
            } else {
                panic!("File: {} Line: {}", file!(), line!())
            }
        });

        let bpe_c = bpe.clone();
        bpe.reset_entry_btn.connect_clicked(move |_| {
            if bpe_c.ok_to_reset_entry() {
                bpe_c.set_edited_paint(None)
            }
        });

        let bpe_c = bpe.clone();
        bpe.new_colln_btn.connect_clicked(move |_| bpe_c.reset());

        let bpe_c = bpe.clone();
        bpe.load_colln_btn
            .connect_clicked(move |_| bpe_c.load_from_file());

        let bpe_c = bpe.clone();
        bpe.save_colln_btn.connect_clicked(move |_| {
            let path = bpe_c
                .saved_file_path()
                .expect("Save requires a saved file path");
            if let Err(err) = bpe_c.write_to_file(&path) {
                bpe_c.report_error("Problem saving file", &err)
            }
        });

        let bpe_c = bpe.clone();
        bpe.save_as_colln_btn.connect_clicked(move |_| {
            if let Err(err) = bpe_c.save_as() {
                bpe_c.report_save_as_failed(&err)
            }
        });

        let bpe_c = bpe.clone();
        bpe.cid_entry
            .connect_changed(move || bpe_c.update_file_button_sensitivities());

        let bpe_c = bpe.clone();
        bpe.file_status_btn.connect_clicked(move |_| {
            let o_path = bpe_c.saved_file_path();
            if let Some(path) = o_path {
                if let Err(err) = bpe_c.write_to_file(&path) {
                    bpe_c.report_error("Problem saving file", &err)
                }
            } else {
                if let Err(err) = bpe_c.save_as() {
                    bpe_c.report_save_as_failed(&err)
                }
            }
        });

        bpe.update_button_sensitivities();

        bpe
    }
}
