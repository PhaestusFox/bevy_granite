use bevy::reflect::TypeRegistry;

use crate::{MetaData, scene::SceneFormatReadDyn};

pub struct EntityDeSerializer<'a, W> {
    type_registry: &'a TypeRegistry,
    stream: &'a mut W,
    indent: usize,
    metadata: &'a MetaData,
    data: &'a mut dyn SceneFormatReadDyn<W>,
}

impl<'a, W> EntityDeSerializer<'a, W> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        stream: &'a mut W,
        indent: usize,
        metadata: &'a MetaData,
        data: &'a mut dyn SceneFormatReadDyn<W>,
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
    pub fn deserialize_entity(&mut self) -> crate::Result<()> {
        let entry = if let Some(end) = find_entry(self.stream) {
            self.stream[1..end - 1].trim()
        } else {
            return Ok(());
        };

        println!("entry: {}", entry);

        Ok(())
    }
}

fn find_entry(str: &str) -> Option<usize> {
    let mut index = 0;
    let mut open = 0;
    let mut in_string = false;
    let mut chars = str.chars();
    while let Some(c) = chars.next() {
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
                    return Some(index);
                }
            }
            '\\' if in_string => {
                chars.next();
            }
            _ => {}
        }
        index += 1;
    }
    None
}
