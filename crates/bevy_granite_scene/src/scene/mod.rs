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
use bevy_granite_core::{EditorIgnore, entities::serialize::SceneMetadata};

use crate::{MetaData, Result};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SceneData {
    pub metadata: SceneMetadata,
    pub entities: Vec<SaveReadyEntity>,
    pub resources: Vec<ResourceSaveReadyData>,
}

impl SceneData {
    pub fn new(entity_count: usize) -> Self {
        Self {
            metadata: SceneMetadata {
                format_version: bevy_granite_core::get_beta_scene_version(),
                entity_count,
            },
            entities: Vec::with_capacity(entity_count),
            resources: Vec::new(),
        }
    }
}

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
}

mod format;

pub use format::*;

mod saver;

pub use saver::SceneSaver;
