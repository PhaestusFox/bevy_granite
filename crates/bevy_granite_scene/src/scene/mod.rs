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
use bevy_granite_core::{EditorIgnore, shared::version::Versions};
use strum::IntoEnumIterator;

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
    ComponentSerializeError(#[from] crate::reflect_serde::ComponentSerializeError),
    #[error("No magic the file is not even 4 bytes long")]
    NoMagic(std::io::Error),
    #[error("Magic does not match expected format")]
    BadMagic([u8; 4]),
    #[error("Missing chapter: [{0}]")]
    MissingChapter(Chapters),
    #[error("Missing field '{1}' in chapter: [{0}]")]
    MissingChapterField(Chapters, &'static str),
    #[error("{0}")]
    VersionError(#[from] bevy_granite_core::shared::version::VersionError),
}

mod format;

pub use format::*;

mod saver;

pub use saver::SceneSaver;

mod loader;

pub use loader::SceneLoader;

pub struct SceneMetadata {
    pub entity_map: EntityHashMap<EntityMetaData>,
    pub uuid_map: Option<HashMap<uuid::Uuid, Entity>>,
    pub version: Versions,
}

impl SceneMetadata {
    pub fn default() -> Self {
        Self {
            entity_map: EntityHashMap::default(),
            uuid_map: None,
            version: Versions::PRE_RELEASE_VERSION,
        }
    }

    pub fn add_entity(&mut self, entity: Entity, meta: EntityMetaData) {
        self.uuid_map
            .get_or_insert_with(HashMap::default)
            .insert(meta.id, entity);
        self.entity_map.insert(entity, meta);
    }
    pub fn extract_from_str(file: &str) -> Result<Self> {
        let start = Chapters::Metadata.find(file)?;
        let end = Chapters::find_next(&file[start..]).unwrap_or(file.len() - start);
        let section = file[start..end]
            .trim()
            .lines()
            .filter(|s| !(s.trim().is_empty() || s.trim_start().starts_with('#')))
            .map(|s| {
                let mut parts = s.split(':');
                let key = parts.next().unwrap_or("").trim();
                let value = parts.next().unwrap_or("").trim().trim_matches(';');
                (key, value)
            })
            .collect::<HashMap<_, _>>();
        let version = section
            .get("version")
            .ok_or(SceneFormatError::MissingChapterField(
                Chapters::Metadata,
                "version",
            ))?
            .trim()
            .parse::<Versions>()?;

        let entity_count = section
            .get("entity_count")
            .and_then(|s| s.trim().parse().ok())
            .map(|c| HashMap::with_capacity(c));
        Ok(Self {
            entity_map: EntityHashMap::new(),
            uuid_map: entity_count,
            version,
        })
    }
}

#[derive(Debug, Clone, Copy, strum_macros::EnumIter)]
pub enum Chapters {
    Metadata,
    Entities,
    Resources,
    NameMap,
    Components,
}

impl std::fmt::Display for Chapters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.header())
    }
}

impl Chapters {
    pub const fn header(&self) -> &'static str {
        match self {
            Chapters::Metadata => "[metadata]",
            Chapters::Entities => "[entities]",
            Chapters::Resources => "[resources]",
            Chapters::NameMap => "[name map]",
            Chapters::Components => "[components]",
        }
    }
    pub const fn header_len(&self) -> usize {
        self.header().len()
    }

    // returns the index of the first character after the chapter header
    pub fn find(&self, haystack: &str) -> Result<usize> {
        haystack
            .find(self.header())
            .map(|i| i + self.header_len())
            .ok_or(SceneFormatError::MissingChapter(*self))
    }

    // returns the index of the start of the next chapter header
    pub fn find_next(haystack: &str) -> Option<usize> {
        for chapter in Chapters::iter() {
            if let Some(index) = haystack.find(chapter.header()) {
                return Some(index);
            }
        }
        None
    }
}

impl AsRef<str> for Chapters {
    fn as_ref(&self) -> &'static str {
        self.header()
    }
}

pub struct ChapterSearcher<'a> {
    haystack: &'a str,
    chapter: Chapters,
    index: usize,
}

impl std::str::pattern::Pattern for Chapters {
    type Searcher<'a> = ChapterSearcher<'a>;

    fn into_searcher(self, haystack: &str) -> Self::Searcher<'_> {
        ChapterSearcher {
            haystack,
            chapter: self,
            index: 0,
        }
    }
}

unsafe impl<'a> std::str::pattern::Searcher<'a> for ChapterSearcher<'a> {
    fn haystack(&self) -> &'a str {
        self.haystack
    }

    fn next(&mut self) -> std::str::pattern::SearchStep {
        let start = self.index;
        if self.haystack[start..].starts_with(self.chapter.header()) {
            self.index += self.chapter.header_len();
            return std::str::pattern::SearchStep::Match(start, start + self.chapter.header_len());
        }
        while self.index + self.chapter.header_len() < self.haystack.len()
            && !self.haystack[self.index..].starts_with(self.chapter.header())
        {
            self.index += 1;
        }
        if self.index + self.chapter.header_len() >= self.haystack.len() {
            std::str::pattern::SearchStep::Done
        } else {
            let match_start = self.index;
            self.index += self.chapter.header_len();
            std::str::pattern::SearchStep::Reject(
                match_start,
                match_start + self.chapter.header_len(),
            )
        }
    }
}
