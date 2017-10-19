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

use std::str::FromStr;

use regex::*;

use paint::PaintError;

trait CharacteristicInterface {
    fn name() -> &'static str;
    fn abbrev(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

// FINISH
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Finish {
    Gloss,
    SemiGloss,
    SemiFlat,
    Flat
}

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
}

lazy_static! {
    pub static ref FINISH_RE: Regex = Regex::new(
        r#"finish\s*=\s*"(?P<finish>\w+)""#
    ).unwrap();
}

impl FromStr for Finish {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<Finish, PaintError> {
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
            _ => Err(PaintError::MalformedText(string.to_string()))
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
            _ => panic!("{:?}: out of bounds Finish", float)
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


// TRANSPARENCY
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Transparency {
    Opaque,
    SemiOpaque,
    SemiTransparent,
    Transparent,
    Clear
}

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
}

lazy_static! {
    pub static ref TRANSPARENCY_RE: Regex = Regex::new(
        r#"transparency\s*=\s*"(?P<transparency>\w+)""#
    ).unwrap();
}

impl FromStr for Transparency {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<Transparency, PaintError> {
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
            _ => Err(PaintError::MalformedText(string.to_string()))
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
            _ => panic!("{:?}: out of bounds Transparency", float)
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


// FLUORESCENCE
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Fluorescence {
    Fluorescent,
    SemiFluorescent,
    SemiNonfluorescent,
    Nonfluorescent
}

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
}

lazy_static! {
    pub static ref FLUORESCENCE_RE: Regex = Regex::new(
        r#"fluorescence\s*=\s*"(?P<fluorescence>\w+)""#
    ).unwrap();
}

impl FromStr for Fluorescence {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<Fluorescence, PaintError> {
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
            _ => Err(PaintError::MalformedText(string.to_string()))
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
            _ => panic!("{:?}: out of bounds Fluorescence", float)
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


// METALLIC
#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum Metallic {
    Metal,
    Metallic,
    SemiMetallic,
    Nonmetallic
}

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
}

lazy_static! {
    pub static ref METALLIC_RE: Regex = Regex::new(
        r#"metallic\s*=\s*"(?P<metallic>\w+)""#
    ).unwrap();
}

impl FromStr for Metallic {
    type Err = PaintError;

    fn from_str(string: &str) -> Result<Metallic, PaintError> {
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
            _ => Err(PaintError::MalformedText(string.to_string()))
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
            _ => panic!("{:?}: out of bounds Metallic", float)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_characteristics_from_str() {
        assert_eq!(Finish::from_str("Flat").unwrap(), Finish::Flat);
        assert_eq!(Finish::from_str(" finish = \"G\"").unwrap(), Finish::Gloss);

        assert_eq!(Transparency::from_str("Opaque").unwrap(), Transparency::Opaque);
        assert_eq!(Transparency::from_str(" transparency = \"ST\"").unwrap(), Transparency::SemiTransparent);

        assert_eq!(Fluorescence::from_str("Fluorescent").unwrap(), Fluorescence::Fluorescent);
        assert_eq!(Fluorescence::from_str(" fluorescence = \"NF\"").unwrap(), Fluorescence::Nonfluorescent);

        assert_eq!(Metallic::from_str("Metal").unwrap(), Metallic::Metal);
        assert_eq!(Metallic::from_str(" metallic = \"NM\"").unwrap(), Metallic::Nonmetallic);
    }
}
