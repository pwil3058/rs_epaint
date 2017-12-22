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
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use pw_gix::colour::*;
use pw_gix::dialogue::*;
use pw_gix::gtkx::notebook::*;

use basic_paint::*;
use super::*;
use super::collection::*;

pub struct CollnPaintCollnBinderCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    notebook: gtk::Notebook,
    paint_selected_callbacks: RefCell<Vec<Box<Fn(&CollnPaint<C, CID>)>>>,
    paint_collns: RefCell<HashMap<Rc<CID>, CollnPaintCollnWidget<A, C, CID>>>,
    paint_colln_files_data_path: PathBuf,
}

impl<A, C, CID> PackableWidgetObject<gtk::Notebook> for CollnPaintCollnBinderCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn pwo(&self) -> gtk::Notebook {
        self.notebook.clone()
    }
}

impl<A, C, CID> CollnPaintCollnBinderCore<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn inform_paint_selected(&self, paint: &CollnPaint<C, CID>) {
        for callback in self.paint_selected_callbacks.borrow().iter() {
            callback(&paint);
        }
    }

    fn remove_paint_colln(&self, ps_id: &CID) {
        let mut paint_collns = self.paint_collns.borrow_mut();
        if let Some(selector) = paint_collns.remove(ps_id) {
            let page_num = self.notebook.page_num(&selector.pwo());
            self.notebook.remove_page(page_num);
        } else {
            panic!("File: {:?} Line: {:?}", file!(), line!())
        }
    }

    pub fn set_target_colour(&self, ocolour: Option<&Colour>) {
        for selector in self.paint_collns.borrow().values() {
            selector.set_target_colour(ocolour);
        }
    }

    pub fn get_colln_file_paths(&self) -> Vec<PathBuf> {
        let mut vpb = Vec::new();
        if !self.paint_colln_files_data_path.exists() {
            return vpb
        };
        let mut file = File::open(&self.paint_colln_files_data_path).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        for line in string.lines() {
            vpb.push(PathBuf::from(line));
        }

        vpb
    }

    pub fn set_colln_file_paths(&self, file_paths: &Vec<PathBuf>) {
        let mut file = File::create(&self.paint_colln_files_data_path).unwrap_or_else(
            |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
        );
        for file_path in file_paths.iter() {
            if let Some(file_path_str) = file_path.to_str() {
                write!(file, "{}\n", file_path_str).unwrap_or_else(
                    |err| panic!("File: {:?} Line: {:?} : {:?}", file!(), line!(), err)
                );
            } else  {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            };
        }
    }

    pub fn connect_paint_selected<F: 'static + Fn(&CollnPaint<C, CID>)>(&self, callback: F) {
        self.paint_selected_callbacks.borrow_mut().push(Box::new(callback))
    }
}

pub type CollnPaintCollnBinder<A, C, CID> = Rc<CollnPaintCollnBinderCore<A, C, CID>>;

pub trait CollnPaintCollnBinderInterface<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn create(data_path: &Path) -> CollnPaintCollnBinder<A, C, CID>;
    fn add_paint_colln(&self, colln_spec: &PaintCollnSpec<C, CID>);
}

impl<A, C, CID> CollnPaintCollnBinderInterface<A, C, CID> for CollnPaintCollnBinder<A, C, CID>
    where   A: ColourAttributesInterface + 'static,
            C: CharacteristicsInterface + 'static,
            CID: CollnIdInterface + 'static,
{
    fn create(data_path: &Path) -> CollnPaintCollnBinder<A, C, CID> {
        let notebook = gtk:: Notebook::new();
        notebook.set_scrollable(true);
        notebook.popup_enable();
        let paint_selected_callbacks: RefCell<Vec<Box<Fn(&CollnPaint<C, CID>)>>> = RefCell::new(Vec::new());
        let paint_collns: RefCell<HashMap<Rc<CID>, CollnPaintCollnWidget<A, C, CID>>> = RefCell::new(HashMap::new());

        let spm = Rc::new(
            CollnPaintCollnBinderCore::<A, C, CID>{
                notebook: notebook,
                paint_selected_callbacks: paint_selected_callbacks,
                paint_collns: paint_collns,
                paint_colln_files_data_path: data_path.to_path_buf(),
            }
        );
        let colln_file_paths = spm.get_colln_file_paths();
        for colln_file_path in colln_file_paths.iter() {
            if let Ok(colln_spec) = PaintCollnSpec::<C, CID>::from_file(&colln_file_path) {
                spm.add_paint_colln(&colln_spec);
            } else {
                let expln = format!("Error parsing \"{:?}\"\n", colln_file_path);
                let msg = "Malformed Paint Series Text";
                warn_user(parent_none(), msg, Some(expln.as_str()));
            }
        };
        spm.notebook.show_all();

        spm
    }

    fn add_paint_colln(&self, colln_spec: &PaintCollnSpec<C, CID>) {
        let mut paint_collns = self.paint_collns.borrow_mut();
        if paint_collns.contains_key(&colln_spec.colln_id) {
            let expln = format!("{} ({}): already included in the tool box.\nSkipped.", colln_spec.colln_id.colln_name(), colln_spec.colln_id.colln_owner());
            inform_user(parent_none(), "Duplicate Paint Series", Some(expln.as_str()));
            return;
        }
        let paint_colln = CollnPaintCollnWidget::<A, C, CID>::create(&colln_spec);
        paint_collns.insert(colln_spec.colln_id.clone(), paint_colln.clone());
        let spm_c = self.clone();
        paint_colln.connect_paint_selected(
            move |paint| spm_c.inform_paint_selected(paint)
        );
        let l_text = format!("{}\n{}", colln_spec.colln_id.colln_name(), colln_spec.colln_id.colln_owner());
        let tt_text = format!("Remove {} ({}) from the tool kit", colln_spec.colln_id.colln_name(), colln_spec.colln_id.colln_owner());
        let label = TabRemoveLabel::create(Some(l_text.as_str()), Some(&tt_text.as_str()));
        let l_text = format!("{} ({})", colln_spec.colln_id.colln_name(), colln_spec.colln_id.colln_owner());
        let menu_label = gtk::Label::new(Some(l_text.as_str()));
        let spm_c = self.clone();
        let ps_id = colln_spec.colln_id.clone();
        label.connect_remove_page(
            move || spm_c.remove_paint_colln(&ps_id)
        );
        self.notebook.append_page_menu(&paint_colln.pwo(), Some(&label.pwo()), Some(&menu_label));
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
