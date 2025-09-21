use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Seek},
};

use super::*;
use crate::scene::SceneFormatError;

pub struct SceneLoader<'a, W> {
    metadata: MetaData,
    register: AppTypeRegistry,
    world: &'a mut World,
    indent: usize,
    format: Box<dyn SceneFormatReadDyn<W>>,
    file: W,
}

impl<'a> SceneLoader<'a, Pointer> {
    pub fn new(world: &'a mut World, file: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let mut file = File::open(file)?;

        let mut buf = [0; 4];
        let mut format = match file.read(&mut buf).map_err(SceneFormatError::NoMagic)? {
            4 => get_format(buf)?,
            _ => {
                return Err(SceneFormatError::NoMagic(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "File too short to contain magic",
                )));
            }
        };
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let meta = MetaData::extract_from_str(data.as_str())?;
        format.extract_str(&data)?;

        // this should run diffrent loaders based on version if compatibile
        if meta.version != bevy_granite_core::get_beta_scene_version() {
            return Err(SceneFormatError::UnsupportedVersion(meta.version));
        }

        Ok(Self {
            metadata: meta,
            file: Pointer::new(data),
            register: world.resource::<AppTypeRegistry>().clone(),
            world,
            format,
            indent: 0,
        })
    }
}

impl<'a> SceneLoader<'a, &str> {
    pub fn load_scene(&mut self) -> crate::Result<()> {
        if self.metadata.uuid_map.capacity() > 0 {
            self.load_entities()?;
        }
        Ok(())
    }

    fn load_entities(&mut self) -> crate::Result<()> {
        let start = self
            .file
            .split("[entities]")
            .nth(1)
            .ok_or(SceneFormatError::MissingSection("entities"))?;
        let ty = self.register.read();

        let mut deserializer = crate::reflect_serializer::EntityDeSerializer::new(
            &ty,
            &mut start,
            self.indent,
            &self.metadata,
            &mut self.format,
        );
        Ok(())
    }

    fn load_resources(&mut self) -> crate::Result<()> {
        Ok(())
    }

    pub(crate) fn crack(self) -> MetaData {
        self.metadata
    }
}

fn get_format<W>(magic: [u8; 4]) -> crate::Result<Box<dyn SceneFormatReadDyn<W>>> {
    match &magic {
        // b"GHSV" => Ok(Box::new(HumanVerbose)),
        b"GHSR" => Ok(Box::new(HumanReduced::default())),
        // b"GHSC" => Ok(Box::new(HumanCompact)),
        // b"GHSB" => Ok(Box::new(Binary)),
        other => Err(SceneFormatError::BadMagic(magic).into()),
    }
}

struct Pointer {
    current: &'static str,
    full: String,
}

impl Pointer {
    fn new(s: String) -> Self {
        Self {
            full: s,
            current: &self.full,
        }
    }
}
