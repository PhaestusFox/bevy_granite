use std::{fmt::Write, io::Read};

use bevy::{
    ecs::{component::Components, entity::EntityHashMap, world},
    prelude::*,
    reflect::TypeRegistry,
};

use bevy_granite_core::EditorIgnore;
use bevy_granite_logging::*;
use serde::Serialize;

use crate::{
    MetaData, Result, pwrite, pwriteln,
    reflect_serializer::{ComponentSerializeError, human_readable::ComponentSerializer},
    scene::{SceneFormatWright, SceneFormatWrightDyn},
};
use crate::{reflect_serializer::pure_reflect::serialize_with_reflect, scene::EntityMetaData};

impl<'a, W: std::fmt::Write> EntitySerializer<'a, W> {
    pub fn serialize_entity(&mut self, entity: Entity, world: &World) -> Result<()> {
        let Some(meta) = self.metadata.entity_map.get(&entity) else {
            return Err(crate::scene::SceneFormatError::EntityNotReserved(entity));
        };
        writeln!(self.stream, "(")?;
        pwriteln!(self.stream, "uuid: {};":self.indent, meta.id)?;
        let entity_data = world.entity(entity);

        // first we ignore all entities that have EditorIgnore::SERIALIZE
        // this indicates the whole entity should not be serialized
        // this should be filtered out by now but might be useful to make entities that have uuids but their data is not serialized
        // for cases like if an entity is linked to another but the other is created only at runtime
        if let Some(ignore) = entity_data.get::<EditorIgnore>()
            && ignore.contains(EditorIgnore::SERIALIZE)
        {
            pwriteln!(self.stream, "# Entity ignored from serialization":self.indent)?;
            pwriteln!(self.stream, ")":self.indent)?;
            return Ok(());
        }
        pwriteln!(self.stream, "[Components]":self.indent)?;
        let mut component_serializer = ComponentSerializer::new(
            self.type_registry,
            self.stream,
            self.indent,
            self.metadata,
            self.data,
        );

        // iterate over all components in the entity
        for component_id in entity_data.archetype().components() {
            // find out the underlying type of the component
            let Some(component_descriptor) = self.components.get_descriptor(component_id) else {
                // skip components that have no descriptor
                // this should never happen because you can't add a component without its descriptor being registered
                continue;
            };
            let r =
                component_serializer.serialize_component(component_descriptor.name(), entity_data);
            if let Err(crate::scene::SceneFormatError::ComponentSerializeError(e)) = r {
                match e {
                    ComponentSerializeError::NoRegistration => {
                        log! {
                            LogType::Editor,
                            LogLevel::Warning,
                            LogCategory::Serialization,
                            "Failed to get registration for component: {0:}\nconsider calling app.register<{0:0}>()", component_descriptor.name()
                        }
                    }
                    ComponentSerializeError::IgnoreSerialize => {}
                    ComponentSerializeError::NoReflectComponent => {
                        log! {
                            LogType::Editor,
                            LogLevel::Warning,
                            LogCategory::Serialization,
                            "Failed to get ReflectComponent for component: {0:}\nconsider adding #[reflect(Component)]", component_descriptor.name()
                        }
                    }
                    ComponentSerializeError::NoComponentData => {
                        log! {
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Serialization,
                            "Failed to get component data for component {0:} on entity {1:}\nThis is a bug, we only attempt to get data for components that are activly present on an entity", component_descriptor.name(), entity
                        };
                    }
                    ComponentSerializeError::DowncastFailed(type_path) => {
                        log! {
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Serialization,
                            "Failed to downcast a instance of {}", type_path
                        };
                        return Err(crate::scene::SceneFormatError::ComponentSerializeError(
                            ComponentSerializeError::DowncastFailed(type_path),
                        ));
                    }
                    ComponentSerializeError::NoTypeInfo(e) => {
                        log! {
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Serialization,
                            "Failed to get TypeInfo for component {0:}: {1:}", component_descriptor.name(), e
                        };
                        return Err(crate::scene::SceneFormatError::ComponentSerializeError(
                            ComponentSerializeError::NoTypeInfo(e),
                        ));
                    }
                    ComponentSerializeError::TypeMissMatch(t, e) => {
                        log! {
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Serialization,
                            "Type miss match for component {0:}, expected it to be of type {1:}", t, e
                        };
                        return Err(crate::scene::SceneFormatError::ComponentSerializeError(
                            ComponentSerializeError::TypeMissMatch(t, e),
                        ));
                    }
                }
            } else {
                r?;
            }
        }

        drop(component_serializer);

        writeln!(self.stream, ")")?;
        Ok(())
    }
}

pub struct EntitySerializer<'a, W> {
    type_registry: &'a TypeRegistry,
    components: &'a Components,
    stream: &'a mut W,
    indent: usize,
    metadata: &'a MetaData,
    data: &'a mut dyn SceneFormatWrightDyn<W>,
}

impl<'a, W: Write> EntitySerializer<'a, W> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        components: &'a Components,
        stream: &'a mut W,
        indent: usize,
        metadata: &'a MetaData,
        data: &'a mut dyn SceneFormatWrightDyn<W>,
    ) -> Self {
        Self {
            type_registry,
            components,
            stream,
            indent,
            metadata,
            data,
        }
    }
}
