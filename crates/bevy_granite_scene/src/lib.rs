#![feature(pattern)]

use bevy::{ecs::entity::EntityHashMap, prelude::*, reflect::TypeRegistry};
use bevy_granite_core::entities::ExposedToEditor;

mod reflect_serde;
mod scene;

type Result<T> = std::result::Result<T, scene::SceneFormatError>;
type MetaData = scene::SceneMetadata;

type Version = bevy_granite_core::shared::version::Versions;

pub struct StrPointer<T> {
    current: usize,
    buffer: T,
}

impl<T> StrPointer<T> {
    fn new(s: T) -> Self {
        Self {
            buffer: s,
            current: 0,
        }
    }
}

impl AsRef<str> for StrPointer<&str> {
    fn as_ref(&self) -> &str {
        &self.buffer[self.current..]
    }
}

impl std::ops::Deref for StrPointer<&str> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::ops::Deref for StrPointer<String> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buffer[self.current..]
    }
}

#[macro_export]
macro_rules! pwrite {
    ($stream:expr, $str:literal:$pad:expr, $($arg:tt)+) => {
        write!($stream, "{}{}", "\t".repeat($pad), format!($str, $($arg)+))
    };
    ($stream:expr, $str:literal:$pad:expr) => {
        write!($stream, "{}{}", "\t".repeat($pad), $str)
    };
    ($stream:expr, $pad:expr) => {
        write!($stream, "{}", "\t".repeat($pad))
    };
}

#[macro_export]
macro_rules! pwriteln {
    ($stream:expr, $str:literal:$pad:expr, $($arg:tt)+) => {
        writeln!($stream, "{}{}", "\t".repeat($pad), format!($str, $($arg)+))
    };
    ($stream:expr, $str:literal:$pad:expr) => {
        writeln!($stream, "{}{}", "\t".repeat($pad), $str)
    };
}
