// Copyright 2018 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gdk_pixbuf;
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
    "                                                                ",
];

pub fn mixtures_print_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(MIXTURES_PRINT_XPM)
}

pub fn mixtures_print_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        mixtures_print_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}
