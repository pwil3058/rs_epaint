// Copyright 2018 Peter Williams <pwil3058@gmail.com>
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

use gdk_pixbuf::{self, PixbufExt};
use gtk;

static MIXTURES_PRINT_XPM: &[&str] = &[
"64 64 4 1",
" 	c None",
".	c #000000",
"+	c #FFFFFF",
"@	c #FD0404",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                ................................                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .+...++++++++++++++++++++++++++.                ",
"                .+...+..++..++++.......++++++++.                ",
"                .++..+++++++++++++++++..+++..++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .+@@@@@.+.+++...+.++..+++++.+++.                ",
"                .+@@@@@...+++++..+++++.++.+++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                                                                ",
"                                                                ",
"      ....................................................      ",
"     ......................................................     ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........................................................    ",
"    ........                                        ........    ",
"    ........                                        ........    ",
"    ........    .++++++++++++++++++++++++++++++.    ........    ",
"    ........    .++++++++++++++++++++++++++++++.    ........    ",
"    ........    .++++++++++++++++++++++++++++++.    ........    ",
"    ........    .++++++++++++++++++++++++++++++.    ........    ",
"     .......    .++++++++++++++++++++++++++++++.    .......     ",
"      ......    .++++++++++++++++++++++++++++++.    ......      ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                .++++++++++++++++++++++++++++++.                ",
"                ................................                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                ",
"                                                                "
];

pub fn mixtures_print_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(MIXTURES_PRINT_XPM)
}

pub fn mixtures_print_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) = mixtures_print_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear) {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}
