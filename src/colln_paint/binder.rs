// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::gtkx::notebook::*;
use pw_gix::recollections::{recall, remember};
use pw_gix::wrapper::*;

use pw_pathux;

use super::collection::*;
use super::*;
use crate::basic_paint::*;

pub struct CollnPaintCollnBinderCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    vbox: gtk::Box,
    notebook: gtk::Notebook,
    load_colln_button: gtk::Button,
    initiate_select_ok: Cell<bool>,
    paint_selected_callbacks: RefCell<Vec<Box<dyn Fn(&CollnPaint<C, CID>)>>>,
    paint_collns: RefCell<Vec<(CollnPaintCollnWidget<A, C, CID>, PathBuf)>>,
    paint_colln_files_data_path: PathBuf,
}

impl_widget_wrapper!(vbox: gtk::Box, CollnPaintCollnBinderCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
);

impl<A, C, CID> CollnPaintCollnBinderCore<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn find_cid(&self, cid: &Rc<CID>) -> Result<usize, usize> {
        self.paint_collns
            .borrow()
            .binary_search_by_key(cid, |colln_data| colln_data.0.colln_id())
    }

    fn find_file_path(&self, path: &Path) -> Option<usize> {
        for (index, colln_data) in self.paint_collns.borrow().iter().enumerate() {
            if path == colln_data.1 {
                return Some(index);
            }
        }
        None
    }

    pub fn set_initiate_select_ok(&self, value: bool) {
        self.initiate_select_ok.set(value);
        for selector in self.paint_collns.borrow().iter() {
            selector.0.set_initiate_select_ok(value);
        }
    }

    fn inform_paint_selected(&self, paint: &CollnPaint<C, CID>) {
        for callback in self.paint_selected_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    fn remove_paint_colln_at_index(&self, index: usize) {
        let selector = self.paint_collns.borrow_mut().remove(index);
        let page_num = self.notebook.page_num(&selector.0.pwo());
        self.notebook.remove_page(page_num);
    }

    fn remove_paint_colln(&self, ps_id: &Rc<CID>) {
        if let Ok(index) = self.find_cid(ps_id) {
            self.remove_paint_colln_at_index(index);
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        for selector in self.paint_collns.borrow().iter() {
            selector.0.set_target_colour(ocolour);
        }
    }

    fn read_colln_file_paths(&self) -> Vec<PathBuf> {
        let mut vpb = Vec::new();
        if !self.paint_colln_files_data_path.exists() {
            return vpb;
        };
        let mut file = File::open(&self.paint_colln_files_data_path)
            .unwrap_or_else(|err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err));
        let mut string = String::new();
        file.read_to_string(&mut string)
            .unwrap_or_else(|err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err));
        for line in string.lines() {
            vpb.push(PathBuf::from(line));
        }

        vpb
    }

    fn write_colln_file_paths(&self) {
        let mut text = String::new();
        for colln_data in self.paint_collns.borrow().iter() {
            text += (pw_pathux::path_to_string(&colln_data.1) + "\n").as_str();
        }
        match File::create(&self.paint_colln_files_data_path) {
            Ok(mut file) => file.write(&text.into_bytes()).unwrap_or_else(|err| {
                panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
            }),
            Err(err) => panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err),
        };
    }

    pub fn connect_paint_selected<F: 'static + Fn(&CollnPaint<C, CID>)>(&self, callback: F) {
        self.paint_selected_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }
}

pub type CollnPaintCollnBinder<A, C, CID> = Rc<CollnPaintCollnBinderCore<A, C, CID>>;

pub trait CollnPaintCollnBinderInterface<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(data_path: &Path) -> CollnPaintCollnBinder<A, C, CID>;
    fn _insert_paint_colln(&self, spec: &PaintCollnSpec<C, CID>, path: &Path, index: usize);
    fn _add_paint_colln_from_file(&self, path: &Path);
    fn load_paint_colln_from_file(&self);
}

impl<A, C, CID> CollnPaintCollnBinderInterface<A, C, CID> for CollnPaintCollnBinder<A, C, CID>
where
    A: ColourAttributesInterface + 'static,
    C: CharacteristicsInterface + 'static,
    CID: CollnIdInterface + 'static,
{
    fn create(data_path: &Path) -> CollnPaintCollnBinder<A, C, CID> {
        let cpcb = Rc::new(CollnPaintCollnBinderCore::<A, C, CID> {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            notebook: gtk::Notebook::new(),
            load_colln_button: gtk::Button::new(),
            initiate_select_ok: Cell::new(false),
            paint_selected_callbacks: RefCell::new(Vec::new()),
            paint_collns: RefCell::new(Vec::new()),
            paint_colln_files_data_path: data_path.to_path_buf(),
        });
        cpcb.notebook.set_scrollable(true);
        cpcb.notebook.popup_enable();

        cpcb.load_colln_button
            .set_tooltip_text(Some("Load collection from file."));
        cpcb.load_colln_button
            .set_image(Some(&CID::colln_load_image(24)));

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&cpcb.load_colln_button, false, true, 2);
        cpcb.vbox.pack_start(&hbox, false, false, 2);
        cpcb.vbox.pack_start(&cpcb.notebook, true, true, 0);

        let colln_file_paths = cpcb.read_colln_file_paths();
        for colln_file_path in colln_file_paths.iter() {
            cpcb._add_paint_colln_from_file(colln_file_path);
        }
        cpcb.write_colln_file_paths();
        cpcb.notebook.show_all();

        let cpcb_c = cpcb.clone();
        cpcb.load_colln_button
            .connect_clicked(move |_| cpcb_c.load_paint_colln_from_file());
        cpcb.vbox.show_all();

        cpcb
    }

    fn _insert_paint_colln(&self, colln_spec: &PaintCollnSpec<C, CID>, path: &Path, index: usize) {
        let mut paint_collns = self.paint_collns.borrow_mut();
        let paint_colln = CollnPaintCollnWidget::<A, C, CID>::create(&colln_spec);
        paint_colln.set_initiate_select_ok(self.initiate_select_ok.get());
        paint_collns.insert(index, (paint_colln.clone(), path.to_path_buf()));
        let cpcb_c = self.clone();
        paint_colln.connect_paint_selected(move |paint| cpcb_c.inform_paint_selected(paint));
        let l_text = format!(
            "{}\n{}",
            colln_spec.colln_id.colln_name(),
            colln_spec.colln_id.colln_owner()
        );
        let tt_text = format!(
            "Remove {} ({}) from the tool kit",
            colln_spec.colln_id.colln_name(),
            colln_spec.colln_id.colln_owner()
        );
        let label = TabRemoveLabel::create(Some(l_text.as_str()), Some(&tt_text.as_str()));
        let l_text = format!(
            "{} ({})",
            colln_spec.colln_id.colln_name(),
            colln_spec.colln_id.colln_owner()
        );
        let menu_label = gtk::Label::new(Some(l_text.as_str()));
        let cpcb_c = self.clone();
        let ps_id = colln_spec.colln_id.clone();
        label.connect_remove_page(move || cpcb_c.remove_paint_colln(&ps_id));
        self.notebook.insert_page_menu(
            &paint_colln.pwo(),
            Some(&label.pwo()),
            Some(&menu_label),
            Some(index as u32),
        );
    }

    fn _add_paint_colln_from_file(&self, path: &Path) {
        match PaintCollnSpec::<C, CID>::from_file(path) {
            Ok(colln_spec) => match self.find_cid(&colln_spec.colln_id) {
                Ok(index) => {
                    let other_file_path = &self.paint_collns.borrow()[index].1;
                    let expln = format!(
                            "\"{}\" ({}): already included in the tool box.\nLoaded from file \"{:?}\".",
                            colln_spec.colln_id.colln_name(),
                            colln_spec.colln_id.colln_owner(),
                            other_file_path,
                        );
                    let buttons = [
                        ("Skip", gtk::ResponseType::Other(0)),
                        ("Replace", gtk::ResponseType::Other(1)),
                    ];
                    if self.ask_question("Duplicate Collection", Some(expln.as_str()), &buttons)
                        == gtk::ResponseType::Other(1)
                    {
                        self.remove_paint_colln_at_index(index);
                        self._insert_paint_colln(&colln_spec, path, index);
                        self.notebook.show_all();
                        self.write_colln_file_paths();
                    }
                }
                Err(index) => {
                    self._insert_paint_colln(&colln_spec, path, index);
                    self.notebook.show_all();
                    self.write_colln_file_paths();
                }
            },
            Err(err) => match err.error_type() {
                &PaintErrorType::IOError(ref io_error) => {
                    let expln = format!("\"{:?}\" \"{}\"\n", path, io_error.description());
                    let msg = "I/O Error";
                    self.warn_user(msg, Some(expln.as_str()));
                }
                &PaintErrorType::MalformedText(_) => {
                    let expln = format!("Error parsing \"{:?}\"\n", path);
                    let msg = "Malformed Collection Specification Text";
                    self.warn_user(msg, Some(expln.as_str()));
                }
                &PaintErrorType::AlreadyExists(ref text) => {
                    let expln = format!("\"{:?}\" contains two paints named\"{}\"\n", path, text);
                    let msg = "Malformed Collection (Duplicate Paints)";
                    self.warn_user(msg, Some(expln.as_str()));
                }
                _ => panic!("File: {} Line: {}: unexpected error."),
            },
        }
    }

    fn load_paint_colln_from_file(&self) {
        let o_last_file = recall(&CID::recollection_name_for("last_colln_loaded_file"));
        let last_file = if let Some(ref text) = o_last_file {
            Some(text.as_str())
        } else {
            None
        };
        if let Some(path) = self.ask_file_path(Some("Collection File Name:"), last_file, true) {
            match pw_pathux::expand_home_dir_or_mine(&path).canonicalize() {
                Ok(abs_file_path) => {
                    if let Some(index) = self.find_file_path(&abs_file_path) {
                        let colln_id = &self.paint_collns.borrow()[index].0.colln_id();
                        let expln = format!(
                            "\"{:?}\": already loaded providing \"{}\" ({}).",
                            path,
                            colln_id.colln_name(),
                            colln_id.colln_owner(),
                        );
                        let buttons = [
                            ("Cancel", gtk::ResponseType::Other(0)),
                            ("Reload", gtk::ResponseType::Other(1)),
                        ];
                        if self.ask_question("Duplicate Collection", Some(expln.as_str()), &buttons)
                            == gtk::ResponseType::Other(1)
                        {
                            self.remove_paint_colln_at_index(index);
                        } else {
                            return;
                        }
                    };
                    self._add_paint_colln_from_file(&abs_file_path);
                    let path_text = pw_pathux::path_to_string(&abs_file_path);
                    remember(
                        &CID::recollection_name_for("last_colln_loaded_file"),
                        &path_text,
                    );
                }
                Err(err) => {
                    let expln = format!("\"{:?}\" \"{}\"\n", path, err.description());
                    let msg = "I/O Error";
                    self.warn_user(msg, Some(expln.as_str()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
