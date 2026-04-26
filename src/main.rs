use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use crate::player::PlayerVelocity;

mod camera;
mod player;
mod world;

fn close_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Oasis Walk".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .init_resource::<PlayerVelocity>()
        .add_systems(
            Startup,
            (world::setup_world, player::spawn_player, camera::spawn_camera),
        )
        .add_systems(
            Update,
            player::setup_player_animation.before(player::update_animation),
        )
        .add_systems(
            Update,
            (
                camera::update_camera,
                player::move_player,
                player::update_player_collider,
                player::update_player_model_offset,
                player::update_animation,
                camera::camera_follow,
            )
                .chain(),
        )
        .add_systems(Update, camera::handle_focus)
        .add_systems(Update, close_on_esc)
        .run();
}
