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

use gdk_pixbuf;
use gtk;

use pw_gix::gdk_pixbufx::PIXOPS_INTERP_BILINEAR;

/* XPM */
static PAINT_STANDARD_XPM: &[&str] = &[
"64 64 10 1",
"0	c #242424",
" 	c None",
"2	c #DB0000",
"3	c #000024",
"4	c #DBDB00",
"5	c #242449",
"6	c #00DBDB",
"7	c #242400",
"8	c #B6926D",
"9	c #499249",
"0000000000000000000000000000                                    ",
"0222222222222222222222222220                                    ",
"0222222222222222222222222220                                    ",
"0222222222222222222222222220                                    ",
"0222222222222222222222222220            333333333333333333333333",
"0222222222222222222222222220            344444444444444444444443",
"0222222222222222222222222220            344444444444444444444443",
"0222222222222222222222222220            344444444444444444444443",
"0222222222222222222222222220            344444444444444444444443",
"0222222222222222222222222220            344444444444444444444443",
"0222222222222222222225555555555555555555355555555555555444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666665444444443",
"0222222222222222222225666666666666666666666666666666663333333333",
"0222222222222222222225666666666666666666666666666666665         ",
"0222222222222222222225666666666666666666666666666666665         ",
"0222222222222222222225666666666666666666666666666666665         ",
"0222222222222222222225666666666667777777777777777777777777      ",
"0222222222222222222225666666666667888888888888888888888887      ",
"0222222222222222222225666666666667888888888888888888888887      ",
"0222222222222233333335666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0222222222222239999995666666666667888888888888888888888887      ",
"0000000000000039999995666666666667888888888888888888888887      ",
"              39999995666666666667888888888888888888888887      ",
"              39999995555555555557888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997888888888888888888888887      ",
"              39999999999999999997777777777777777777777777      ",
"              399999999999999999999993                          ",
"              399999999999999999999993                          ",
"              399999999999999999999993                          ",
"              399999999999999999999993                          ",
"              399999999999999999999993                          ",
"              399999999999999999999993                          ",
"   3333 333333399339999339993393333393   33    33333   33333    ",
"  3        3  333333333333333333333333   33    33  3   33   33  ",
"  3        3    33 3   333  33 33    3  33 3   33  33  33    3  ",
"   333     3    3  3   3  3 33 33    3  3  3   33  3   33    3  ",
"     33    3    3  3   3  3 33 33    3  3  3   33333   33    3  ",
"      33   3   333333  3   333 33    3 333333  33  3   33    3  ",
"      33   3   3    3  3   333 33   33 3    3  33   3  33   33  ",
"  33333    3   3    33 3    33 33333   3    33 33   3  33333    ",
"                                                                "
];

pub fn paint_standard_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(PAINT_STANDARD_XPM)
}

pub fn paint_standard_image(size: i32) -> gtk::Image {
    if let Ok(pixbuf) = paint_standard_pixbuf().scale_simple(size, size, PIXOPS_INTERP_BILINEAR) {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}

/* XPM */
static PAINT_STANDARD_LOAD_XPM: &[&str] = &[
"64 64 12 1",
"0	c #242424",
" 	c None",
"2	c #DB0000",
"3	c #004900",
"4	c #00DB00",
"5	c #000024",
"6	c #DBDB00",
"7	c #242449",
"8	c #00DBDB",
"9	c #242400",
"A	c #B6926D",
"B	c #499249",
"0000000000000000000000000000                                    ",
"0222222222222222222222222220                                    ",
"0222222222222222222222222220             3333333333             ",
"0222222222222222222222222220             3444444443             ",
"0222222222222222222222222220            534444444435555555555555",
"0222222222222222222222222220            534444444436666666666665",
"0222222222222222222222222220            534444444436666666666665",
"0222222222222222222222222220            534444444436666666666665",
"0222222222222222222222222220            534444444436666666666665",
"0222222222222222222222222220            534444444436666666666665",
"0222222222222222222227777777777777777777534444444437777666666665",
"0222222222222222222227888888888888888888834444444438887666666665",
"0222222222222222222227888888888888888888834444444438887666666665",
"0222222222222222222227888888888888888888834444444438887666666665",
"0222222222222222222227888888888888888888834444444438887666666665",
"0222222222222222222227888888883333333333334444444433333333333335",
"0222222222222222222227888888883444444444444444444444444444444435",
"0222222222222222222227888888883444444444444444444444444444444435",
"022222222222222222222788888888344444444444444444444444444444443 ",
"022222222222222222222788888888344444444444444444444444444444443 ",
"022222222222222222222788888888344444444444444444444444444444443 ",
"022222222222222222222788888888344444444444444444444444444444443 ",
"022222222222222222222788888888344444444444444444444444444444443 ",
"022222222222222222222788888888333333333333444444443333333333333 ",
"0222222222222255555557888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"022222222222225BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"000000000000005BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"              5BBBBBB7888888888889AAAAAAA3444444443AAAAAA9      ",
"              5BBBBBB7777777777779AAAAAAA3333333333AAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9AAAAAAAAAAAAAAAAAAAAAAA9      ",
"              5BBBBBBBBBBBBBBBBBB9999999999999999999999999      ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"              5BBBBBBBBBBBBBBBBBBBBBB5                          ",
"   5555 5555555BB55BBBB55BBB55B55555B5   55    55555   55555    ",
"  5        5  555555555555555555555555   55    55  5   55   55  ",
"  5        5    55 5   555  55 55    5  55 5   55  55  55    5  ",
"   555     5    5  5   5  5 55 55    5  5  5   55  5   55    5  ",
"     55    5    5  5   5  5 55 55    5  5  5   55555   55    5  ",
"      55   5   555555  5   555 55    5 555555  55  5   55    5  ",
"      55   5   5    5  5   555 55   55 5    5  55   5  55   55  ",
"  55555    5   5    55 5    55 55555   5    55 55   5  55555    ",
"                                                                "
];

pub fn paint_standard_load_pixbuf() -> gdk_pixbuf::Pixbuf {
    gdk_pixbuf::Pixbuf::new_from_xpm_data(PAINT_STANDARD_LOAD_XPM)
}

pub fn paint_standard_load_image(size: i32) -> gtk::Image {
    if let Ok(pixbuf) = paint_standard_load_pixbuf().scale_simple(size, size, PIXOPS_INTERP_BILINEAR) {
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    } else {
        panic!("File: {:?} Line: {:?}", file!(), line!())
    }
}