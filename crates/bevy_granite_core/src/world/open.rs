use std::io::BufRead;

use crate::absolute_asset_to_rel;
use crate::events::{RequestLoadEvent, WorldLoadSuccessEvent};
use crate::shared::version::{self, Version};
use crate::{assets::AvailableEditableMaterials, entities::deserialize_scene_v0_1_4};
use bevy::prelude::*;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

/// Watches for RequestLoadEvent and then deserializes the world from its path
pub fn open_world_reader(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    mut world_open_reader: MessageReader<RequestLoadEvent>,
    mut world_load_success_writer: MessageWriter<WorldLoadSuccessEvent>,
) {
    if let Some(RequestLoadEvent(path, save_settings, translation)) =
        world_open_reader.read().next()
    {
        let rel = absolute_asset_to_rel(path.to_string()).to_string();
        // this is temp and is about checking version of scene file to then call correct deserializer
        // this is just because i dont know what path im supposed to expect
        let abs_path: std::borrow::Cow<'static, str> = crate::rel_asset_to_absolute(&rel);
        let file = match std::fs::File::open(abs_path.as_ref()) {
            Ok(file) => file,
            Err(e) => {
                log!(
                    LogType::Game,
                    LogLevel::Error,
                    LogCategory::System,
                    "Failed to open file {}: {}. Are you sure it exists?",
                    path,
                    e
                );
                return;
            }
        };
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            // I dont know how a buffer read can fail but bail if it does
            let Ok(line) = line else {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::System,
                    "Failed to read line in file {}",
                    &rel
                );
                return;
            };
            // if line doesnt contain version info skip
            if !line.contains("version:") {
                continue;
            }
            let version = line
                .split("version:")
                .nth(1)
                .unwrap()
                .trim()
                .trim_start_matches('"')
                .trim_end_matches(',')
                .trim_end_matches('"');

            match version.parse::<Version>() {
                // if version is 0.1.4 call old code
                Ok(Version::V0_1_4) => {
                    deserialize_scene_v0_1_4(
                        &asset_server,
                        &mut commands,
                        &mut materials,
                        &mut available_materials,
                        meshes,
                        rel.clone(),
                        save_settings.clone(),
                        *translation,
                    );
                    log!(
                        LogType::Game,
                        LogLevel::OK,
                        LogCategory::System,
                        "Loaded world: {:?}",
                        &rel
                    );
                    world_load_success_writer.write(WorldLoadSuccessEvent(rel.clone()));
                    return;
                }
                // if is supported version we will call new code when it exists
                Ok(v) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Error,
                        LogCategory::System,
                        "Unsupported version {:?} in file {}",
                        v,
                        &rel
                    );
                    return;
                }
                // if unsupported version format bail
                Err(e) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Error,
                        LogCategory::System,
                        "Failed to parse version in file {}: {}",
                        &rel,
                        e
                    );
                    return;
                }
            };
        }
    }
}
