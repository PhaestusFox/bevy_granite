use bevy::{
    asset::uuid,
    ecs::{
        component::{ComponentId, Components},
        entity::{Entity, EntityHashMap, EntityHashSet},
        reflect::AppTypeRegistry,
        world::World,
    },
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::TypeRegistry,
};
use bevy_granite_core::EditorIgnore;

use crate::{MetaData, Result};

pub struct EntityMetaData {
    pub id: uuid::Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SaveReadyEntity {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ResourceSaveReadyData {}

#[derive(thiserror::Error, Debug)]
pub enum SceneFormatError {
    #[error("Unknown format: {0}")]
    UnknownFormat(String),
    #[error("Format write error: {0}")]
    FmtWriteError(#[from] std::fmt::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error(
        "Entity {0:?} has no metadata, this means it doesn't have a associated UUID, this might be automatic in future but for now all entities must be reserved before serializing"
    )]
    EntityNotReserved(Entity),
    #[error("component serialization error: {0}")]
    ComponentSerializeError(#[from] crate::reflect_serializer::ComponentSerializeError),
    #[error("No magic the file is not even 4 bytes long")]
    NoMagic(std::io::Error),
    #[error("Magic does not match expected format")]
    BadMagic([u8; 4]),
    #[error("Missing section: [{0}]")]
    MissingSection(&'static str),
    #[error("Missing field '{1}' in section: [{0}]")]
    MissingSectionField(&'static str, &'static str),
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(String),
}

mod format;

pub use format::*;

mod saver;

pub use saver::SceneSaver;

mod loader;

pub use loader::SceneLoader;

pub struct SceneMetadata {
    pub entity_map: EntityHashMap<EntityMetaData>,
    pub uuid_map: HashMap<uuid::Uuid, Entity>,
    pub version: String,
}

impl SceneMetadata {
    pub fn default() -> Self {
        Self {
            entity_map: EntityHashMap::default(),
            uuid_map: HashMap::default(),
            version: bevy_granite_core::get_beta_scene_version(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity, meta: EntityMetaData) {
        self.uuid_map.insert(meta.id, entity);
        self.entity_map.insert(entity, meta);
    }
    pub fn extract_from_str(file: &str) -> Result<Self> {
        let start = file
            .split("[metadata]")
            .nth(1)
            .ok_or(SceneFormatError::MissingSection("metadata"))?;
        let meta = start
            .split('[')
            .next()
            .expect("at least one section")
            .trim();
        let section = meta
            .lines()
            .filter(|s| !(s.trim().is_empty() || s.trim_start().starts_with('#')))
            .map(|s| {
                let mut parts = s.split(':');
                let key = parts.next().unwrap_or("").trim();
                let value = parts.next().unwrap_or("").trim().trim_matches(';');
                (key, value)
            })
            .collect::<HashMap<_, _>>();
        let Some(version) = section.get("format_version") else {
            return Err(SceneFormatError::MissingSectionField(
                "metadata",
                "format_version",
            ));
        };

        let entity_count = section
            .get("entity_count")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        Ok(Self {
            entity_map: EntityHashMap::new(),
            uuid_map: HashMap::with_capacity(entity_count),
            version: version.to_string(),
        })
    }
}
