use bevy::{
    platform::collections::{HashMap, HashSet},
    reflect::{Reflect, TypeRegistry},
};

pub trait SceneFormat<W> {
    fn magic() -> [u8; 4];
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

pub trait SceneFormatDyn<W> {
    fn magic(&self) -> [u8; 4];
    fn get_component_display_name<'a>(&'a mut self, type_path: &'static str) -> &'a str;
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()>;
    fn add_tail(&self, wrighter: &mut W) -> crate::Result<()>;
}

impl<W, T: SceneFormat<W>> SceneFormatDyn<W> for T {
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
impl<W: std::fmt::Write> SceneFormat<W> for HumanVerbose {
    fn magic() -> [u8; 4] {
        *b"GHSV"
    }
    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: HumanVerbose;")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct HumanReduced {
    used_names: HashSet<String>,
    long_name_mapping: HashMap<&'static str, String>,
}

impl<W: std::fmt::Write> SceneFormat<W> for HumanReduced {
    fn magic() -> [u8; 4] {
        *b"GHSR"
    }

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
        data.long_name_mapping.entry(type_path).or_insert_with(|| {
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
pub struct HumanCompact;
impl<W: std::fmt::Write> SceneFormat<W> for HumanCompact {
    fn magic() -> [u8; 4] {
        *b"GHSC"
    }

    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: HumanCompact;")?;
        Ok(())
    }
}
pub struct Binary;
impl<W: std::io::Write> SceneFormat<W> for Binary {
    fn magic() -> [u8; 4] {
        *b"GBS_"
    }

    fn add_head(&self, wrighter: &mut W) -> crate::Result<()> {
        writeln!(wrighter, "format: Binary;")?;
        Ok(())
    }
}
