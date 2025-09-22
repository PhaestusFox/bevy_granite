use super::TransformGizmo;
use crate::{
    gizmos::{GizmoOf, GizmoSnap},
    input::GizmoAxis,
    selection::{ActiveSelection, Selected},
    GizmoCamera, RequestDuplicateEntityEvent,
};
use bevy::{
    asset::Assets,
    ecs::{
        component::Component, event::EventWriter, hierarchy::ChildOf, observer::Trigger,
        system::Commands,
    },
    gizmos::{retained::Gizmo, GizmoAsset},
    picking::events::{Drag, DragStart, Pointer, Pressed},
    prelude::{Entity, GlobalTransform, Query, Res, ResMut, Transform, Vec3, With, Without},
};
use bevy_granite_core::UserInput;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn drag_transform_gizmo(
    event: Trigger<Pointer<Drag>>,
    targets: Query<&GizmoOf>,
    camera_query: Query<
        (Entity, &GlobalTransform, &bevy::render::camera::Camera),
        With<GizmoCamera>,
    >,
    mut objects: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    other_selected: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    gizmo_snap: Res<GizmoSnap>,
    gizmo_data: Query<(&GizmoAxis, &TransformGizmo)>,
    user_input: Res<UserInput>,
) {
    // Only drag with Primary Input drags
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    let Ok((axis, typ)) = gizmo_data.get(event.target) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.target
        );
        return;
    };

    let Ok((c_entity, camera_transform, camera)) = camera_query.single() else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo camera not found",
        };
        return;
    };

    let Ok(GizmoOf(target)) = targets.get(event.target) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo target not found for entity {:?}",
            event.target
        };
        return;
    };
    let Ok(click_ray) = camera.viewport_to_world(camera_transform, event.pointer_location.position)
    else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Failed to convert viewport to world coordinates for pointer location: {:?}",
            event.pointer_location.position
        };
        return;
    };

    // Get all selected entities (active + other selected)
    let mut all_selected_entities = Vec::new();
    all_selected_entities.extend(active_selection.iter());
    all_selected_entities.extend(other_selected.iter());

    // Filter out entities that are children of other selected entities
    // (non-active children move with their parents automatically)
    let mut root_entities = Vec::new();
    for &entity in &all_selected_entities {
        let mut is_child_of_selected = false;
        if let Ok(parent) = parents.get(entity) {
            if all_selected_entities.contains(&parent.parent()) {
                is_child_of_selected = true;
            }
        }
        if !is_child_of_selected {
            root_entities.push(entity);
        }
    }

    if root_entities.is_empty() {
        log! {
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "No root entities to transform"
        };
        return;
    }

    // Get the current world position for calculations
    let current_world_pos = {
        let Ok(target_transform) = objects.get(*target) else {
            log! {
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Input,
                "Gizmo target transform not found for entity {:?}",
                target
            };
            return;
        };

        if let Ok(global_transform) = global_transforms.get(*target) {
            global_transform.translation()
        } else {
            target_transform.translation
        }
    };

    // Calculate the world delta based on the gizmo axis and type
    let world_delta = match (axis, typ) {
        (GizmoAxis::None, _) => Vec3::ZERO,
        (GizmoAxis::X, TransformGizmo::Axis) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., current_world_pos.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() + (click_ray.direction * click_distance);
            let delta_x = snap_gizmo(hit.x, gizmo_snap.transform_value) - current_world_pos.x;
            Vec3::new(delta_x, 0.0, 0.0)
        }
        (GizmoAxis::Y, TransformGizmo::Axis) => {
            let mut normal = camera_transform.forward().as_vec3();
            normal.y = 0.0;
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(current_world_pos.x, 0., current_world_pos.z),
                bevy::math::primitives::InfinitePlane3d::new(normal.normalize()),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            let delta_y = snap_gizmo(hit.y, gizmo_snap.transform_value) - current_world_pos.y;
            Vec3::new(0.0, delta_y, 0.0)
        }
        (GizmoAxis::Z, TransformGizmo::Axis) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., current_world_pos.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            let delta_z = snap_gizmo(hit.z, gizmo_snap.transform_value) - current_world_pos.z;
            Vec3::new(0.0, 0.0, delta_z)
        }
        (GizmoAxis::X, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(current_world_pos.x, 0., 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::X),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            let delta_y = snap_gizmo(hit.y, gizmo_snap.transform_value) - current_world_pos.y;
            let delta_z = snap_gizmo(hit.z, gizmo_snap.transform_value) - current_world_pos.z;
            Vec3::new(0.0, delta_y, delta_z)
        }
        (GizmoAxis::Y, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., current_world_pos.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            let delta_x = snap_gizmo(hit.x, gizmo_snap.transform_value) - current_world_pos.x;
            let delta_z = snap_gizmo(hit.z, gizmo_snap.transform_value) - current_world_pos.z;
            Vec3::new(delta_x, 0.0, delta_z)
        }
        (GizmoAxis::Z, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., 0., current_world_pos.z),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Z),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            let delta_x = snap_gizmo(hit.x, gizmo_snap.transform_value) - current_world_pos.x;
            let delta_y = snap_gizmo(hit.y, gizmo_snap.transform_value) - current_world_pos.y;
            Vec3::new(delta_x, delta_y, 0.0)
        }
        (GizmoAxis::All, _) => {
            let camera_right = camera_transform.rotation() * Vec3::X;
            let camera_up = camera_transform.rotation() * Vec3::Y;
            let movement_scale = 0.005;
            let world_delta =
                (camera_right * event.delta.x + camera_up * -event.delta.y) * movement_scale;
            Vec3::new(
                snap_gizmo(world_delta.x, gizmo_snap.transform_value),
                snap_gizmo(world_delta.y, gizmo_snap.transform_value),
                snap_gizmo(world_delta.z, gizmo_snap.transform_value),
            )
        }
    };

    // Apply the delta to all root selected entities
    for &entity in &root_entities {
        if let Ok(mut entity_transform) = objects.get_mut(entity) {
            if let Ok(parent) = parents.get(entity) {
                if let Ok(parent_global) = global_transforms.get(parent.parent()) {
                    let parent_rotation_inv =
                        parent_global.to_scale_rotation_translation().1.inverse();
                    let parent_local_delta = parent_rotation_inv * world_delta;
                    entity_transform.translation += parent_local_delta;
                } else {
                    entity_transform.translation += world_delta;
                }
            } else {
                entity_transform.translation += world_delta;
            }
        }
    }
    
    if user_input.ctrl_left.any {
        if let Ok(mut camera_transform) = objects.get_mut(c_entity) {
            camera_transform.translation += world_delta;
        }
    }
}

pub fn dragstart_transform_gizmo(
    event: Trigger<Pointer<DragStart>>,
    targets: Query<&GizmoOf>,
    gizmo_data: Query<(&GizmoAxis, &TransformGizmo)>,
    user_input: Res<UserInput>,
    mut dispatch: EventWriter<RequestDuplicateEntityEvent>,
) {
    if user_input.mouse_middle.any || !user_input.shift_left.pressed {
        return;
    }
    let Ok(_) = gizmo_data.get(event.target) else {
        return;
    };
    let Ok(GizmoOf(target)) = targets.get(event.target) else {
        return;
    };
    log!("Attempting Drag Duplicate");
    dispatch.write(RequestDuplicateEntityEvent {
        entity: target.clone(),
    });
}

fn snap_gizmo(value: f32, inc: f32) -> f32 {
    if inc == 0.0 {
        value
    } else {
        (value / inc).round() * inc
    }
}

pub fn draw_axis_lines(
    event: Trigger<Pointer<Pressed>>,
    gizmo_data: Query<(&GizmoAxis, &GizmoOf, &TransformGizmo), With<TransformGizmo>>,
    mut bevy_gizmo: ResMut<Assets<GizmoAsset>>,
    mut commands: Commands,
    origin: Query<&GlobalTransform>,
) {
    let Ok((axis, root, transform)) = gizmo_data.get(event.target) else {
        return;
    };
    if let GizmoAxis::All = axis {
        return;
    }
    let Ok(origin) = origin.get(root.get()) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo origin transform not found for entity {:?}",
            root.0
        };
        return;
    };
    let mut asset = GizmoAsset::new();
    match transform {
        TransformGizmo::Axis => {
            asset.line(
                origin.translation() + axis.to_vec3() * 1000.,
                origin.translation() + axis.to_vec3() * -1000.,
                axis.color(),
            );
        }
        TransformGizmo::Plane => {
            let (a, b) = axis.plane();
            asset.line(
                origin.translation() + a.to_vec3() * 1000.,
                origin.translation() + a.to_vec3() * -1000.,
                a.color(),
            );
            asset.line(
                origin.translation() + b.to_vec3() * 1000.,
                origin.translation() + b.to_vec3() * -1000.,
                b.color(),
            );
        }
    }

    commands.spawn((
        *axis,
        GizmoOf(root.0),
        Gizmo {
            handle: bevy_gizmo.add(asset),
            ..Default::default()
        },
        AxisLine,
    ));
}

pub fn cleanup_axis_line(
    mut commands: Commands,
    query: Query<Entity, With<AxisLine>>,
    user_input: Res<UserInput>,
) {
    if user_input.mouse_left.just_released {
        for entity in query.iter() {
            commands.entity(entity).try_despawn();
        }
    }
}

#[derive(Component)]
pub struct AxisLine;
