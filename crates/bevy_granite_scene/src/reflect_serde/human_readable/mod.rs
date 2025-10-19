mod component_serializer;
mod entity_serializer;
mod reflect_serializer;

pub use component_serializer::ComponentSerializer;
pub use entity_serializer::EntitySerializer;
pub use reflect_serializer::ReflectSerializer;

mod entity_deserializer;
pub use entity_deserializer::EntityDeSerializer;

// this is just the raw data inside the secene file so i have something to output
pub struct TempEntityData {
    pub raw: String,
}
