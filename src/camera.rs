use crate::player::{CAMERA_CROUCH_OFFSET, CAMERA_FPS_HEIGHT, Player, PlayerState};
use avian3d::prelude::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

const CAMERA_SENSITIVITY: f32 = 0.006;
const CAMERA_TPS_HEIGHT: f32 = 5.0;
const CAMERA_INITIAL_DISTANCE: f32 = 5.0;
const CAMERA_INITIAL_YAW: f32 = std::f32::consts::PI;
const CAMERA_INITIAL_PITCH: f32 = 0.4;
const CAMERA_WALL_MIN_DISTANCE: f32 = 0.3;
const CAMERA_WALL_CLAMP_MIN: f32 = 0.1;
const CAMERA_PITCH_MIN: f32 = -1.19;
const CAMERA_PITCH_MAX: f32 = 1.4;
const CAMERA_DISTANCE_MIN: f32 = 2.0;
const CAMERA_DISTANCE_MAX: f32 = 100.0;


#[derive(Component, PartialEq, Clone, Copy)]
pub enum CameraMode {
    TPS,
    FPS,
}

#[derive(Component)]
pub struct CameraAngle {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for CameraAngle {
    fn default() -> Self {
        Self {
            yaw: CAMERA_INITIAL_YAW,
            pitch: CAMERA_INITIAL_PITCH,
            distance: CAMERA_INITIAL_DISTANCE,
        }
    }
}

impl CameraAngle {
    pub fn add_yaw(&mut self, delta: f32) {
        self.yaw -= delta;
    }
    pub fn add_pitch(&mut self, delta: f32) {
        self.pitch = (self.pitch - delta).clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);
    }
    pub fn add_distance(&mut self, delta: f32) {
        self.distance = (self.distance - delta).clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
    }
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    pub fn pitch(&self) -> f32 {
        self.pitch
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
}

/// Spawn the camera entity with TPS mode and default angle.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraMode::TPS,
        CameraAngle::default(),
    ));
}

/// Handle camera mode toggle (V kye), mouse look, and scroll zoom.
pub fn update_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut camera_query: Query<(&mut CameraMode, &mut CameraAngle), With<Camera3d>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok((mut mode, mut angle)) = camera_query.single_mut() else {
        return;
    };

    // Toggle TPS / FPS
    if keyboard.just_pressed(KeyCode::KeyV) {
        *mode = match *mode {
            CameraMode::TPS => {
                if let Ok(player_transform) = player_query.single() {
                    let (yaw, _, _) = player_transform.rotation.to_euler(EulerRot::YXZ);
                    angle.yaw = yaw + std::f32::consts::PI;
                    angle.pitch = 0.0;
                }
                CameraMode::FPS
            }
            CameraMode::FPS => {
                if let Ok(player_transform) = player_query.single() {
                    let (yaw, _, _) = player_transform.rotation.to_euler(EulerRot::YXZ);
                    angle.yaw = yaw + std::f32::consts::PI;
                    angle.pitch = CAMERA_INITIAL_PITCH;
                }
                CameraMode::TPS
            }
        };
    }

    let sensitivity = CAMERA_SENSITIVITY;
    angle.add_yaw(mouse_motion.delta.x * sensitivity);
    angle.add_pitch(mouse_motion.delta.y * sensitivity);

    angle.add_distance(mouse_scroll.delta.y);
}

/// Position the camera relative to the player each frame.
/// TPS: orbit camera with wall collision. FPS: attach to player head.
pub fn camera_follow(
    player_query: Query<(Entity, &Transform, &PlayerState), With<Player>>,
    mut camera_query: Query<
        (&mut Transform, &CameraMode, &CameraAngle),
        (With<Camera3d>, Without<Player>),
    >,
    spatial_query: SpatialQuery,
) {
    let Ok((player_entity, player_transform, player_state)) = player_query.single() else {
        return;
    };
    let Ok((mut camera_transform, mode, angle)) = camera_query.single_mut() else {
        return;
    };

    let crouch_offset = match *player_state {
        PlayerState::CrouchIdle | PlayerState::CrouchWalking => CAMERA_CROUCH_OFFSET,
        _ => 0.0,
    };

    let rotation = Quat::from_rotation_y(angle.yaw()) * Quat::from_rotation_x(angle.pitch());

    match *mode {
        CameraMode::TPS => {
            let ideal_offset = rotation * Vec3::new(0.0, CAMERA_TPS_HEIGHT + crouch_offset, angle.distance());
            let ideal_pos = player_transform.translation + ideal_offset;

            // Raycast from player to ideal camera position to detect walls
            let Ok(direction) = Dir3::new(ideal_offset.normalize()) else { return; };
            let distance = ideal_offset.length();

            let actual_pos = match spatial_query.cast_ray(
                player_transform.translation,
                direction,
                distance,
                true,
                &SpatialQueryFilter::from_excluded_entities(vec![player_entity]),
            ) {
                Some(hit) => {
                    // Wall hit: place camera slightly in front of the wall
                    player_transform.translation
                        + ideal_offset.normalize() * (hit.distance - CAMERA_WALL_MIN_DISTANCE).max(CAMERA_WALL_CLAMP_MIN)
                }
                None => ideal_pos, 
            };

            camera_transform.translation = actual_pos;
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
        CameraMode::FPS => {
            let offset = Vec3::new(0.0, CAMERA_FPS_HEIGHT + crouch_offset, 0.0);
            let forward = rotation * Vec3::new(0.0, 0.0, -0.15);
            camera_transform.translation = player_transform.translation + offset + forward;
            camera_transform.rotation = rotation;
        }
    }
}

/// Lock cursor on left click, release on focus loss (e.g. Alt + Tab)
pub fn handle_focus(
    window: Single<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if window.focused && mouse_button.just_pressed(MouseButton::Left) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    } else if !window.focused {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}
