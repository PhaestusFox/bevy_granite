use bevy::{ecs::entity::Entity, reflect::TypeRegistry};

use crate::{
    MetaData, Result, StrPointer,
    reflect_serde::human_readable::TempEntityData as EntityData,
    scene::{Chapters, SceneFormatReadDyn},
};

pub struct EntityDeSerializer<'a, W> {
    type_registry: &'a TypeRegistry,
    stream: StrPointer<W>,
    indent: usize,
    metadata: &'a MetaData,
    data: &'a crate::scene::BasicFormats,
}

impl<'a, W> EntityDeSerializer<'a, W> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        stream: StrPointer<W>,
        indent: usize,
        metadata: &'a MetaData,
        data: &'a crate::scene::BasicFormats,
    ) -> Self {
        Self {
            type_registry,
            stream,
            indent,
            metadata,
            data,
        }
    }
}
impl<'a> EntityDeSerializer<'a, &'a str> {
    pub fn deserialize_entity(&mut self) -> crate::Result<Option<(Entity, EntityData)>> {
        let Some((start, end)) = find_entry(&self.stream) else {
            return Ok(None);
        };
        let out = self.stream[start..end].trim();
        let data = EntityData {
            raw: out.to_string(),
        };
        self.stream.current += end;
        Ok(Some((Entity::PLACEHOLDER, data)))
    }
}

// todo make &str generic
impl<'a> EntityDeSerializer<'a, &'a str> {
    pub fn drain(&mut self) -> Vec<Result<(Entity, EntityData)>> {
        let mut start = self.stream.current;
        let end = Chapters::find_next(self.stream.buffer).unwrap_or(self.stream.len());

        let mut entries = Vec::new();
        while self.stream.current < end {
            match self.deserialize_entity() {
                Ok(Some(v)) => entries.push(Ok(v)),
                Ok(None) => break,
                Err(e) => entries.push(Err(e)),
            }
        }
        entries
    }
}

fn find_entry(str: &str) -> Option<(usize, usize)> {
    let mut index = 0;
    let mut open = 0;
    let mut start = 0;
    let mut in_string = false;
    let mut chars = str.chars();
    while let Some(c) = chars.next() {
        index += 1;
        if open == 0 && c.is_whitespace() {
            start += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = !in_string;
            }
            '(' if !in_string => {
                open += 1;
            }
            ')' if !in_string => {
                open -= 1;
                if open == 0 {
                    println!("Found entry from {} to {}", start, index);
                    return Some((start, index));
                }
            }
            '\\' if in_string => {
                chars.next();
            }
            _ => {}
        }
    }
    None
}
