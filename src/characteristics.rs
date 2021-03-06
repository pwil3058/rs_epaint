// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;

use pw_gix::gtk::{self, prelude::ComboBoxExtManual, ComboBoxTextExt};

use regex::*;

#[derive(Debug)]
pub struct CharacteristicError {
    msg: String,
}

impl CharacteristicError {
    pub fn new(text: &str) -> CharacteristicError {
        CharacteristicError {
            msg: format!("{}: is a malformed characteristic value.", text),
        }
    }
}

impl fmt::Display for CharacteristicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CharacteristicError({}).", self.to_string())?;
        Ok(())
    }
}

impl Error for CharacteristicError {
    fn description(&self) -> &str {
        &self.msg
    }
}

pub trait CharacteristicInterface: FromStr + PartialEq {
    fn name() -> &'static str;
    fn abbrev(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn values() -> &'static [Self];

    fn prompt() -> String {
        Self::name().to_string() + ":"
    }
}

pub trait CharacteristicEntryInterface {
    type Characteristic: CharacteristicInterface + 'static;

    fn combo_box_text(&self) -> gtk::ComboBoxText;
    fn create() -> Self;

    fn get_value(&self) -> Option<Self::Characteristic> {
        if let Some(text) = self.combo_box_text().get_active_text() {
            match Self::Characteristic::from_str(&text) {
                Ok(value) => Some(value),
                Err(_) => panic!(
                    "File: {:?} Line: {:?} illegal value: {:?}",
                    file!(),
                    line!(),
                    text
                ),
            }
        } else {
            None
        }
    }

    fn set_value(&self, o_value: Option<Self::Characteristic>) {
        if let Some(value) = o_value {
            for (i, f_value) in Self::Characteristic::values().iter().enumerate() {
                if value == *f_value {
                    self.combo_box_text().set_active(Some(i as u32));
                    break;
                }
            }
        } else {
            self.combo_box_text().set_active(None);
        }
    }
}

macro_rules! implement_entry_core {
    ( $characteristic:ident, $entry_core:ident ) => {
        pub struct $entry_core {
            combo_box_text: gtk::ComboBoxText,
        }

        impl CharacteristicEntryInterface for Rc<$entry_core> {
            type Characteristic = $characteristic;

            fn combo_box_text(&self) -> gtk::ComboBoxText {
                self.combo_box_text.clone()
            }

            fn create() -> Rc<$entry_core> {
                let combo_box_text = gtk::ComboBoxText::new();
                for value in $characteristic::values().iter() {
                    combo_box_text.append_text(value.description());
                }
                combo_box_text.set_active(Some(0));
                Rc::new($entry_core { combo_box_text })
            }
        }
    };
}

// FINISH
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat,
}

static FINISH_VALUES: &[Finish] = &[
    Finish::Gloss,
    Finish::SemiGloss,
    Finish::SemiFlat,
    Finish::Flat,
];

impl Finish {}

impl CharacteristicInterface for Finish {
    fn name() -> &'static str {
        "Finish"
    }

    fn abbrev(&self) -> &'static str {
        match *self {
            Finish::Gloss => "G",
            Finish::SemiGloss => "SG",
            Finish::SemiFlat => "SF",
            Finish::Flat => "F",
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Finish::Gloss => "Gloss",
            Finish::SemiGloss => "Semi-gloss",
            Finish::SemiFlat => "Semi-flat",
            Finish::Flat => "Flat",
        }
    }

    fn values() -> &'static [Finish] {
        FINISH_VALUES
    }
}

lazy_static! {
    pub static ref FINISH_RE: Regex = Regex::new(r#"finish\s*=\s*"(?P<finish>\w+)""#).unwrap();
}

impl FromStr for Finish {
    type Err = CharacteristicError;

    fn from_str(string: &str) -> Result<Finish, CharacteristicError> {
        let mut mstr = string;
        if let Some(c) = FINISH_RE.captures(string) {
            if let Some(m) = c.name("finish") {
                mstr = m.as_str()
            }
        }
        match mstr {
            "G" | "Gloss" => Ok(Finish::Gloss),
            "SG" | "Semi-gloss" => Ok(Finish::SemiGloss),
            "SF" | "Semi-flat" => Ok(Finish::SemiFlat),
            "F" | "Flat" => Ok(Finish::Flat),
            _ => Err(CharacteristicError::new(string)),
        }
    }
}

impl From<f64> for Finish {
    fn from(float: f64) -> Finish {
        match float.round() as u8 {
            4 => Finish::Gloss,
            3 => Finish::SemiGloss,
            2 => Finish::SemiFlat,
            1 => Finish::Flat,
            _ => panic!("{:?}: out of bounds Finish", float),
        }
    }
}

impl From<Finish> for f64 {
    fn from(finish: Finish) -> f64 {
        match finish {
            Finish::Gloss => 4.0,
            Finish::SemiGloss => 3.0,
            Finish::SemiFlat => 2.0,
            Finish::Flat => 1.0,
        }
    }
}

impl fmt::Display for Finish {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "finish=\"{}\"", self.abbrev())
    }
}

implement_entry_core!(Finish, FinishEntryCore);

pub type FinishEntry = Rc<FinishEntryCore>;

// TRANSPARENCY
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Transparency {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    Transparent,
    Clear,
}

static TRANSPARENCY_VALUES: &[Transparency] = &[
    Transparency::Opaque,
    Transparency::SemiOpaque,
    Transparency::SemiTransparent,
    Transparency::Transparent,
    Transparency::Clear,
];

impl CharacteristicInterface for Transparency {
    fn name() -> &'static str {
        "Transparency"
    }

    fn abbrev(&self) -> &'static str {
        match *self {
            Transparency::Opaque => "O",
            Transparency::SemiOpaque => "SO",
            Transparency::SemiTransparent => "ST",
            Transparency::Transparent => "T",
            Transparency::Clear => "Cl",
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Transparency::Opaque => "Opaque",
            Transparency::SemiOpaque => "Semi-opaque",
            Transparency::SemiTransparent => "Semi-transparent",
            Transparency::Transparent => "Transparent",
            Transparency::Clear => "Clear",
        }
    }

    fn values() -> &'static [Transparency] {
        TRANSPARENCY_VALUES
    }
}

lazy_static! {
    pub static ref TRANSPARENCY_RE: Regex =
        Regex::new(r#"transparency\s*=\s*"(?P<transparency>\w+)""#).unwrap();
}

impl FromStr for Transparency {
    type Err = CharacteristicError;

    fn from_str(string: &str) -> Result<Transparency, CharacteristicError> {
        let mut mstr = string;
        if let Some(c) = TRANSPARENCY_RE.captures(string) {
            if let Some(m) = c.name("transparency") {
                mstr = m.as_str()
            }
        }
        match mstr {
            "O" | "Opaque" => Ok(Transparency::Opaque),
            "SO" | "Semi-opaque" => Ok(Transparency::SemiOpaque),
            "ST" | "Semi-transparent" => Ok(Transparency::SemiTransparent),
            "T" | "Transparent" => Ok(Transparency::Transparent),
            "Cl" | "Clear" => Ok(Transparency::Clear),
            _ => Err(CharacteristicError::new(string)),
        }
    }
}

impl From<f64> for Transparency {
    fn from(float: f64) -> Transparency {
        match float.round() as u8 {
            5 => Transparency::Opaque,
            4 => Transparency::SemiOpaque,
            3 => Transparency::SemiTransparent,
            2 => Transparency::Transparent,
            1 => Transparency::Clear,
            _ => panic!("{:?}: out of bounds Transparency", float),
        }
    }
}

impl From<Transparency> for f64 {
    fn from(finish: Transparency) -> f64 {
        match finish {
            Transparency::Opaque => 5.0,
            Transparency::SemiOpaque => 4.0,
            Transparency::SemiTransparent => 3.0,
            Transparency::Transparent => 2.0,
            Transparency::Clear => 1.0,
        }
    }
}

impl fmt::Display for Transparency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transparency=\"{}\"", self.abbrev())
    }
}

implement_entry_core!(Transparency, TransparencyEntryCore);

pub type TransparencyEntry = Rc<TransparencyEntryCore>;

// PERMANENCE
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Permanence {
    ExtremelyPermanent,
    Permanent,
    ModeratelyDurable,
    Fugitive,
}

static PERMANENCE_VALUES: &[Permanence] = &[
    Permanence::ExtremelyPermanent,
    Permanence::Permanent,
    Permanence::ModeratelyDurable,
    Permanence::Fugitive,
];

impl CharacteristicInterface for Permanence {
    fn name() -> &'static str {
        "Permanence"
    }

    fn abbrev(&self) -> &'static str {
        match *self {
            Permanence::ExtremelyPermanent => "AA",
            Permanence::Permanent => "A",
            Permanence::ModeratelyDurable => "B",
            Permanence::Fugitive => "T",
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Permanence::ExtremelyPermanent => "Extremely Permanent",
            Permanence::Permanent => "Permanent",
            Permanence::ModeratelyDurable => "Moderately Durable",
            Permanence::Fugitive => "Fugitive",
        }
    }

    fn values() -> &'static [Permanence] {
        PERMANENCE_VALUES
    }
}

lazy_static! {
    pub static ref PERMANENCE_RE: Regex =
        Regex::new(r#"permanence\s*=\s*"(?P<permanence>\w+)""#).unwrap();
}

impl FromStr for Permanence {
    type Err = CharacteristicError;

    fn from_str(string: &str) -> Result<Permanence, CharacteristicError> {
        let mut mstr = string;
        if let Some(c) = PERMANENCE_RE.captures(string) {
            if let Some(m) = c.name("permanence") {
                mstr = m.as_str()
            }
        }
        match mstr {
            "AA" | "Extremely Permanent" => Ok(Permanence::ExtremelyPermanent),
            "A" | "Permanent" => Ok(Permanence::Permanent),
            "B" | "Moderately Durable" => Ok(Permanence::ModeratelyDurable),
            "T" | "Fugitive" => Ok(Permanence::Fugitive),
            _ => Err(CharacteristicError::new(string)),
        }
    }
}

impl From<f64> for Permanence {
    fn from(float: f64) -> Permanence {
        match float.round() as u8 {
            4 => Permanence::ExtremelyPermanent,
            3 => Permanence::Permanent,
            2 => Permanence::ModeratelyDurable,
            1 => Permanence::Fugitive,
            _ => panic!("{:?}: out of bounds Permanence", float),
        }
    }
}

impl From<Permanence> for f64 {
    fn from(finish: Permanence) -> f64 {
        match finish {
            Permanence::ExtremelyPermanent => 4.0,
            Permanence::Permanent => 3.0,
            Permanence::ModeratelyDurable => 2.0,
            Permanence::Fugitive => 1.0,
        }
    }
}

impl fmt::Display for Permanence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "permanence=\"{}\"", self.abbrev())
    }
}

implement_entry_core!(Permanence, PermanenceEntryCore);

pub type PermanenceEntry = Rc<PermanenceEntryCore>;

// FLUORESCENCE
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Fluorescence {
    Fluorescent,
    SemiFluorescent,
    SemiNonfluorescent,
    Nonfluorescent,
}

static FLUORENCE_VALUES: &[Fluorescence] = &[
    Fluorescence::Fluorescent,
    Fluorescence::SemiFluorescent,
    Fluorescence::SemiNonfluorescent,
    Fluorescence::Nonfluorescent,
];

impl CharacteristicInterface for Fluorescence {
    fn name() -> &'static str {
        "Fluorescence"
    }

    fn abbrev(&self) -> &'static str {
        match *self {
            Fluorescence::Fluorescent => "Fl",
            Fluorescence::SemiFluorescent => "SF",
            Fluorescence::SemiNonfluorescent => "SN",
            Fluorescence::Nonfluorescent => "NF",
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Fluorescence::Fluorescent => "Fluorescent",
            Fluorescence::SemiFluorescent => "Semi-fluorescent",
            Fluorescence::SemiNonfluorescent => "Semi-nonfluorescent",
            Fluorescence::Nonfluorescent => "Nonfluorescent",
        }
    }

    fn values() -> &'static [Fluorescence] {
        FLUORENCE_VALUES
    }
}

lazy_static! {
    pub static ref FLUORESCENCE_RE: Regex =
        Regex::new(r#"fluorescence\s*=\s*"(?P<fluorescence>\w+)""#).unwrap();
}

impl FromStr for Fluorescence {
    type Err = CharacteristicError;

    fn from_str(string: &str) -> Result<Fluorescence, CharacteristicError> {
        let mut mstr = string;
        if let Some(c) = FLUORESCENCE_RE.captures(string) {
            if let Some(m) = c.name("fluorescence") {
                mstr = m.as_str()
            }
        }
        match mstr {
            "Fl" | "Fluorescent" => Ok(Fluorescence::Fluorescent),
            "SF" | "Semi-fluorescent" => Ok(Fluorescence::SemiFluorescent),
            "SN" | "Semi-nonfluorescent" => Ok(Fluorescence::SemiNonfluorescent),
            "NF" | "Nonfluorescent" => Ok(Fluorescence::Nonfluorescent),
            _ => Err(CharacteristicError::new(string)),
        }
    }
}

impl From<f64> for Fluorescence {
    fn from(float: f64) -> Fluorescence {
        match float.round() as u8 {
            4 => Fluorescence::Fluorescent,
            3 => Fluorescence::SemiFluorescent,
            2 => Fluorescence::SemiNonfluorescent,
            1 => Fluorescence::Nonfluorescent,
            _ => panic!("{:?}: out of bounds Fluorescence", float),
        }
    }
}

impl From<Fluorescence> for f64 {
    fn from(finish: Fluorescence) -> f64 {
        match finish {
            Fluorescence::Fluorescent => 4.0,
            Fluorescence::SemiFluorescent => 3.0,
            Fluorescence::SemiNonfluorescent => 2.0,
            Fluorescence::Nonfluorescent => 1.0,
        }
    }
}

impl fmt::Display for Fluorescence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fluorescence=\"{}\"", self.abbrev())
    }
}

implement_entry_core!(Fluorescence, FluorescenceEntryCore);

pub type FluorescenceEntry = Rc<FluorescenceEntryCore>;

// METALLIC
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Metallic {
    Metal,
    Metallic,
    SemiMetallic,
    Nonmetallic,
}

static METALLIC_VALUES: &[Metallic] = &[
    Metallic::Metal,
    Metallic::Metallic,
    Metallic::SemiMetallic,
    Metallic::Nonmetallic,
];

impl CharacteristicInterface for Metallic {
    fn name() -> &'static str {
        "Metallic"
    }

    fn abbrev(&self) -> &'static str {
        match *self {
            Metallic::Metal => "Ml",
            Metallic::Metallic => "Mc",
            Metallic::SemiMetallic => "SM",
            Metallic::Nonmetallic => "NM",
        }
    }

    fn description(&self) -> &'static str {
        match *self {
            Metallic::Metal => "Metal",
            Metallic::Metallic => "Semi-metallic",
            Metallic::SemiMetallic => "Semi-nonmetallic",
            Metallic::Nonmetallic => "Nonmetallic",
        }
    }

    fn values() -> &'static [Metallic] {
        METALLIC_VALUES
    }
}

lazy_static! {
    pub static ref METALLIC_RE: Regex =
        Regex::new(r#"metallic\s*=\s*"(?P<metallic>\w+)""#).unwrap();
}

impl FromStr for Metallic {
    type Err = CharacteristicError;

    fn from_str(string: &str) -> Result<Metallic, CharacteristicError> {
        let mut mstr = string;
        if let Some(c) = METALLIC_RE.captures(string) {
            if let Some(m) = c.name("metallic") {
                mstr = m.as_str()
            }
        }
        match mstr {
            "Ml" | "Metal" => Ok(Metallic::Metal),
            "Mc" | "Semi-metallic" => Ok(Metallic::Metallic),
            "SM" | "Semi-nonmetallic" => Ok(Metallic::SemiMetallic),
            "NM" | "Nonmetallic" => Ok(Metallic::Nonmetallic),
            _ => Err(CharacteristicError::new(string)),
        }
    }
}

impl From<f64> for Metallic {
    fn from(float: f64) -> Metallic {
        match float.round() as u8 {
            4 => Metallic::Metal,
            3 => Metallic::Metallic,
            2 => Metallic::SemiMetallic,
            1 => Metallic::Nonmetallic,
            _ => panic!("{:?}: out of bounds Metallic", float),
        }
    }
}

impl From<Metallic> for f64 {
    fn from(finish: Metallic) -> f64 {
        match finish {
            Metallic::Metal => 4.0,
            Metallic::Metallic => 3.0,
            Metallic::SemiMetallic => 2.0,
            Metallic::Nonmetallic => 1.0,
        }
    }
}

impl fmt::Display for Metallic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "metallic=\"{}\"", self.abbrev())
    }
}

implement_entry_core!(Metallic, MetallicEntryCore);

pub type MetallicEntry = Rc<MetallicEntryCore>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_characteristics_from_str() {
        assert_eq!(Finish::from_str("Flat").unwrap(), Finish::Flat);
        assert_eq!(Finish::from_str(" finish = \"G\"").unwrap(), Finish::Gloss);

        assert_eq!(
            Transparency::from_str("Opaque").unwrap(),
            Transparency::Opaque
        );
        assert_eq!(
            Transparency::from_str(" transparency = \"ST\"").unwrap(),
            Transparency::SemiTransparent
        );

        assert_eq!(
            Fluorescence::from_str("Fluorescent").unwrap(),
            Fluorescence::Fluorescent
        );
        assert_eq!(
            Fluorescence::from_str(" fluorescence = \"NF\"").unwrap(),
            Fluorescence::Nonfluorescent
        );

        assert_eq!(Metallic::from_str("Metal").unwrap(), Metallic::Metal);
        assert_eq!(
            Metallic::from_str(" metallic = \"NM\"").unwrap(),
            Metallic::Nonmetallic
        );
    }
}
