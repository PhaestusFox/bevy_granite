use std::{borrow::Cow, io::Read};

use bevy::{
    platform::collections::{HashMap, HashSet},
    reflect::{Reflect, TypeRegistry},
};

use crate::scene::SceneFormatError;

pub trait SceneFormatWright<W>: SceneMagic {
    #[allow(unused_variables)]
    fn get_component_display_name<'a>(data: &'a mut Self, type_path: &'static str) -> &'a str {
        type_path
    }
    #[allow(unused_variables)]
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn add_tail(&self, wrighter: &mut W) -> crate::Result<()> {
        Ok(())
    }
}

pub trait SceneFormatWrightDyn<W> {
    fn magic(&self) -> [u8; 4];
    fn get_component_display_name<'a>(&'a mut self, type_path: &'static str) -> &'a str;
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()>;
    fn add_tail(&self, wrighter: &mut W) -> crate::Result<()>;
}

pub trait SceneFormatRead<W>: SceneMagic {
    #[allow(unused_variables)]
    fn extract_str(&mut self, reader: &str) -> crate::Result<()> {
        Ok(())
    }
}

pub trait SceneFormatReadDyn<W> {
    fn magic(&self) -> [u8; 4];
    fn extract_str(&mut self, reader: &str) -> crate::Result<()>;
}

impl<W, T: SceneFormatRead<W>> SceneFormatReadDyn<W> for T {
    fn magic(&self) -> [u8; 4] {
        T::magic()
    }
    fn extract_str(&mut self, reader: &str) -> crate::Result<()> {
        T::extract_str(self, reader)
    }
}

impl<W, T: SceneFormatWright<W>> SceneFormatWrightDyn<W> for T {
    fn magic(&self) -> [u8; 4] {
        T::magic()
    }
    fn get_component_display_name<'a>(&'a mut self, type_path: &'static str) -> &'a str {
        T::get_component_display_name(self, type_path)
    }
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        self.add_head(wrighter)
    }
    fn add_tail(&self, wrighter: &mut W) -> crate::Result<()> {
        self.add_tail(wrighter)
    }
}

#[derive(Default)]
pub struct HumanVerbose;

impl SceneMagic for HumanVerbose {
    fn magic() -> [u8; 4] {
        *b"GHSV"
    }
}

impl<W: std::fmt::Write> SceneFormatWright<W> for HumanVerbose {
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: HumanVerbose;")?;
        Ok(())
    }
}

impl<W: std::io::Read> SceneFormatRead<W> for HumanVerbose {}

#[derive(Default)]
pub struct HumanReduced {
    used_names: HashSet<String>,
    long_name_mapping: HashMap<Cow<'static, str>, String>,
}

impl SceneMagic for HumanReduced {
    fn magic() -> [u8; 4] {
        *b"GHSR"
    }
}

impl<W: std::fmt::Write> SceneFormatWright<W> for HumanReduced {
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: HumanReduced;")?;
        Ok(())
    }

    fn add_tail(&self, wrighter: &mut W) -> crate::Result<()> {
        if !self.used_names.is_empty() {
            writeln!(wrighter, "[Name Map]")?;
            for (long, short) in self.long_name_mapping.iter() {
                writeln!(wrighter, "{short} -> {long};")?;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn get_component_display_name<'a>(data: &'a mut Self, type_path: &'static str) -> &'a str {
        data.long_name_mapping
            .entry(type_path.into())
            .or_insert_with(|| {
                let mut start = type_path.rfind("::").map(|i| i + 2).unwrap_or(0);
                let mut short = &type_path[start..];
                while data.used_names.contains(short) {
                    start = type_path[..start].rfind("::").map(|i| i + 2).unwrap_or(0);
                    short = &type_path[start..];
                }
                data.used_names.insert(short.to_string());
                short.to_string()
            })
    }
}

// impl SceneFormatRead<std::fs::File> for HumanReduced {
//     fn extract_str(&mut self, reader: &mut std::fs::File) -> crate::Result<()> {
//         // would be better if we could search in place
//         let mut data = String::new();
//         reader.read_to_string(&mut data)?;

//         let start = data
//             .split("[Name Map]")
//             .nth(1)
//             .ok_or(SceneFormatError::MissingSection("Name Map"))?;
//         let end = start.find("[").unwrap_or(start.len());
//         let names = &start[..end];
//         for line in names.lines() {
//             let line = line.trim();
//             if line.is_empty() {
//                 continue;
//             }
//             let Some((short, long)) = line.split_once("->") else {
//                 continue;
//             };
//             let short = short.trim();
//             let long = long.trim().trim_end_matches(';');
//             self.long_name_mapping
//                 .insert(long.to_string().into(), short.to_string());
//         }
//         Ok(())
//     }
// }

impl SceneFormatRead<&str> for HumanReduced {
    fn extract_str(&mut self, reader: &str) -> crate::Result<()> {
        let start = reader
            .find("[Name Map]")
            .ok_or(SceneFormatError::MissingSection("Name Map"))?;
        let end = reader.find("[").unwrap_or(reader.len());
        let names = &reader[start..end];
        for line in names.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((short, long)) = line.split_once("->") else {
                continue;
            };
            let short = short.trim();
            let long = long.trim().trim_end_matches(';');
            self.long_name_mapping
                .insert(long.to_string().into(), short.to_string());
        }
        Ok(())
    }
}

pub struct HumanCompact;

impl SceneMagic for HumanCompact {
    fn magic() -> [u8; 4] {
        *b"GHSC"
    }
}

impl<W: std::fmt::Write> SceneFormatWright<W> for HumanCompact {
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: HumanCompact;")?;
        Ok(())
    }
}

impl<W: std::io::Read> SceneFormatRead<W> for HumanCompact {}

pub struct Binary;

impl SceneMagic for Binary {
    fn magic() -> [u8; 4] {
        *b"GBSV"
    }
}

impl<W: std::fmt::Write> SceneFormatWright<W> for Binary {
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: BinaryVerbose;")?;
        Ok(())
    }
}

impl<W: std::io::Read> SceneFormatRead<W> for Binary {}

pub trait SceneMagic {
    fn magic() -> [u8; 4];
}
