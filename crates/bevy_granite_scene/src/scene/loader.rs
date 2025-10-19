use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Seek},
};

use super::*;
use crate::{StrPointer, scene::SceneFormatError};

pub struct SceneLoader<'a, W> {
    metadata: MetaData,
    register: AppTypeRegistry,
    world: &'a mut World,
    indent: usize,
    format: super::format::BasicFormats,
    file: StrPointer<W>,
}

impl<'a> SceneLoader<'a, String> {
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
        if meta.version != Version::PRE_RELEASE_VERSION {
            return Err(SceneFormatError::VersionError(
                bevy_granite_core::shared::version::VersionError::InvalidVersion(
                    meta.version.to_string(),
                ),
            ));
        }

        Ok(Self {
            metadata: meta,
            file: crate::StrPointer::new(data),
            register: world.resource::<AppTypeRegistry>().clone(),
            world,
            format,
            indent: 0,
        })
    }
}

impl<'a> SceneLoader<'a, String> {
    pub fn load_scene(&mut self) -> crate::Result<()> {
        if self.metadata.uuid_map.is_some() {
            self.load_entities()?;
        }
        Ok(())
    }

    fn load_entities(&mut self) -> crate::Result<()> {
        let start = Chapters::Entities.find(&self.file)?;
        let ty = self.register.read();

        let mut deserializer = crate::reflect_serde::EntityDeSerializer::new(
            &ty,
            crate::StrPointer::new(&self.file.buffer[start..]),
            self.indent,
            &self.metadata,
            &self.format,
        );
        for entry in deserializer.drain() {
            let (entity, data) = entry?;
            println!("entity: {:?}\n\n", entity);
        }
        Ok(())
    }

    fn load_resources(&mut self) -> crate::Result<()> {
        Ok(())
    }

    pub(crate) fn crack(self) -> MetaData {
        self.metadata
    }
}

impl<'a, T> SceneLoader<'a, T> {
    pub fn format(&self) -> &super::format::BasicFormats {
        &self.format
    }
}

// fn get_format<W>(magic: [u8; 4]) -> crate::Result<Box<dyn SceneFormatReadDyn<W>>> {
//     match &magic {
//         // b"GHSV" => Ok(Box::new(HumanVerbose)),
//         b"GHSR" => Ok(Box::new(HumanReduced::default())),
//         // b"GHSC" => Ok(Box::new(HumanCompact)),
//         // b"GHSB" => Ok(Box::new(Binary)),
//         other => Err(SceneFormatError::BadMagic(magic).into()),
//     }
// }

fn get_format(magic: [u8; 4]) -> crate::Result<super::format::BasicFormats> {
    match &magic {
        b"GHSV" => Ok(super::format::BasicFormats::HumanVerbose),
        b"GHSR" => Ok(super::format::BasicFormats::HumanReduced(HashMap::new())),
        // b"GHSC" => Ok(Box::new(HumanCompact)),
        // b"GHSB" => Ok(Box::new(Binary)),
        other => Err(SceneFormatError::BadMagic(magic).into()),
    }
}
