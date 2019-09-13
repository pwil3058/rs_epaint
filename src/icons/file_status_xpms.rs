// Copyright 2018 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gdk_pixbuf;
use gtk;

static NEEDS_SAVE_NOT_READY_XPM: &[&str] = &[
    "64 64 2 1",
    " 	c None",
    "1	c #FF0000",
    "                          111111111111                          ",
    "                      11111111111111111111                      ",
    "                    111111111111111111111111                    ",
    "                  1111111111111111111111111111                  ",
    "                11111111111111111111111111111111                ",
    "              111111111111111111111111111111111111              ",
    "             11111111111111111111111111111111111111             ",
    "            1111111111111111111111111111111111111111            ",
    "          11111111111111111111111111111111111111111111          ",
    "         1111111111111111111111111111111111111111111111         ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "       11111111111111111111111111111111111111111111111111       ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "       11111111111111111111111111111111111111111111111111       ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "         1111111111111111111111111111111111111111111111         ",
    "          11111111111111111111111111111111111111111111          ",
    "            1111111111111111111111111111111111111111            ",
    "             11111111111111111111111111111111111111             ",
    "              111111111111111111111111111111111111              ",
    "                11111111111111111111111111111111                ",
    "                  1111111111111111111111111111                  ",
    "                    111111111111111111111111                    ",
    "                      11111111111111111111                      ",
    "                          111111111111                          ",
];

pub fn needs_save_not_ready_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(NEEDS_SAVE_NOT_READY_XPM)
}

pub fn needs_save_not_ready_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        needs_save_not_ready_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

static NEEDS_SAVE_READY_XPM: &[&str] = &[
    "64 64 2 1",
    " 	c None",
    "1	c #FFAF00",
    "                          111111111111                          ",
    "                      11111111111111111111                      ",
    "                    111111111111111111111111                    ",
    "                  1111111111111111111111111111                  ",
    "                11111111111111111111111111111111                ",
    "              111111111111111111111111111111111111              ",
    "             11111111111111111111111111111111111111             ",
    "            1111111111111111111111111111111111111111            ",
    "          11111111111111111111111111111111111111111111          ",
    "         1111111111111111111111111111111111111111111111         ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "       11111111111111111111111111111111111111111111111111       ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "       11111111111111111111111111111111111111111111111111       ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "         1111111111111111111111111111111111111111111111         ",
    "          11111111111111111111111111111111111111111111          ",
    "            1111111111111111111111111111111111111111            ",
    "             11111111111111111111111111111111111111             ",
    "              111111111111111111111111111111111111              ",
    "                11111111111111111111111111111111                ",
    "                  1111111111111111111111111111                  ",
    "                    111111111111111111111111                    ",
    "                      11111111111111111111                      ",
    "                          111111111111                          ",
];

pub fn needs_save_ready_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(NEEDS_SAVE_READY_XPM)
}

pub fn needs_save_ready_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        needs_save_ready_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

static UP_TO_DATE_XPM: &[&str] = &[
    "64 64 2 1",
    " 	c None",
    "1	c #00FF00",
    "                          111111111111                          ",
    "                      11111111111111111111                      ",
    "                    111111111111111111111111                    ",
    "                  1111111111111111111111111111                  ",
    "                11111111111111111111111111111111                ",
    "              111111111111111111111111111111111111              ",
    "             11111111111111111111111111111111111111             ",
    "            1111111111111111111111111111111111111111            ",
    "          11111111111111111111111111111111111111111111          ",
    "         1111111111111111111111111111111111111111111111         ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "       11111111111111111111111111111111111111111111111111       ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "1111111111111111111111111111111111111111111111111111111111111111",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    " 11111111111111111111111111111111111111111111111111111111111111 ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "  111111111111111111111111111111111111111111111111111111111111  ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "   1111111111111111111111111111111111111111111111111111111111   ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "    11111111111111111111111111111111111111111111111111111111    ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "     111111111111111111111111111111111111111111111111111111     ",
    "      1111111111111111111111111111111111111111111111111111      ",
    "       11111111111111111111111111111111111111111111111111       ",
    "        111111111111111111111111111111111111111111111111        ",
    "        111111111111111111111111111111111111111111111111        ",
    "         1111111111111111111111111111111111111111111111         ",
    "          11111111111111111111111111111111111111111111          ",
    "            1111111111111111111111111111111111111111            ",
    "             11111111111111111111111111111111111111             ",
    "              111111111111111111111111111111111111              ",
    "                11111111111111111111111111111111                ",
    "                  1111111111111111111111111111                  ",
    "                    111111111111111111111111                    ",
    "                      11111111111111111111                      ",
    "                          111111111111                          ",
];

pub fn up_to_date_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(UP_TO_DATE_XPM)
}

pub fn up_to_date_image(size: i32) -> gtk::Image {
    if let Some(pixbuf) =
        up_to_date_pixbuf().scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
    {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}
