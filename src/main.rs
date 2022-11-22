#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::result;

use bevy::{prelude::*, transform};
use bevy::window::PresentMode;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_inspector_egui::{WorldInspectorPlugin, InspectorPlugin, Inspectable};
use rand::prelude::*;

pub const WINDOW_WIDTH: f32 = 1600.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            title: "Boids".to_string(),
            present_mode: PresentMode::AutoNoVsync,
            resizable: true,
            ..Default::default()
        },
        ..default()
    }))

    .register_type::<Settings>()
    .register_type::<Direction>()

    .insert_resource(Settings::default())
    
    .add_startup_system(spawn_camera)
    .add_startup_system(spawn_boids)

    .add_system(flocking_system)
    .add_system(resize_system)
    .add_system(movement_system)
    .add_system(wrap_borders_system)
    
    .add_plugin(InspectorPlugin::<Settings>::new())
    .add_plugin(WorldInspectorPlugin::new())
    
    .add_plugin(LogDiagnosticsPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())

    .run()
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_boids(
    mut commands: Commands
) {
    for _i in 0..200 {
        let mut rng = thread_rng();
        let pos_x: f32 = rng.gen_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let pos_y: f32 = rng.gen_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
        let dir_x: f32 = rng.gen_range(-1.0..1.0);
        let dir_y: f32 = rng.gen_range(-1.0..1.0);
        
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(10.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(pos_x, pos_y, 0.0),
            ..default()
        })
        .insert(Direction(Vec2::new(dir_x, dir_y).normalize()));
    }
}

fn flocking_system(
    mut query: Query<(&GlobalTransform, &mut Direction)>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    let average_dir = Vec2::new(0.0, 0.0);
    let mut combinations = query.iter_combinations_mut();
    while let Some([(t1, mut dir1), (t2, dir2)]) = combinations.fetch_next() {
        if t1.translation().distance(t2.translation()) < settings.vision_distance {
            // Cohesion
            dir1.0 = dir1.0.rotate(Vec2::from_angle(t1.translation().angle_between(t2.translation()) * time.delta_seconds() * settings.alignment));

            // Alignment
            dir1.0 = dir1.0.rotate(Vec2::from_angle(dir1.0.angle_between(dir2.0) * time.delta_seconds() * settings.alignment));
        }
    }
}

fn resize_system(
    mut objects: Query<&mut Transform, With<Direction>>,
    settings: Res<Settings>
) {
    for mut transform in &mut objects {
        transform.scale.x = settings.size;
        transform.scale.y = settings.size;
    }
}

fn movement_system(
    mut objects: Query<(&Direction, &mut Transform)>,
    time: Res<Time>,
    settings: Res<Settings>
){
    if !settings.paused {
        for (direction, mut transform) in &mut objects {
            transform.translation.x += direction.0.x * time.delta_seconds() * settings.move_speed;
            transform.translation.y += direction.0.y * time.delta_seconds() * settings.move_speed;
        }
    }
}

fn wrap_borders_system(
    mut objects: Query<&mut Transform, With<Direction>>,
    windows: ResMut<Windows>
) {
    let window = windows.get_primary().unwrap();
    let width = window.width();
    let height = window.height();

    for mut transform in &mut objects {
        if transform.translation.x >= width / 2.0 {
            transform.translation.x = -width / 2.0 + 1.0;
        } else if transform.translation.x <= -width / 2.0 {
            transform.translation.x = width / 2.0 - 1.0;
        }
        if transform.translation.y >= height / 2.0 {
            transform.translation.y = -height / 2.0 + 1.0;
        } else if transform.translation.y <= -height / 2.0 {
            transform.translation.y = height / 2.0 - 1.0;
        }
    }
}

#[derive(Reflect, Resource, Inspectable)]
pub struct Settings {
    move_speed: f32,
    vision_distance: f32,
    size: f32,
    separation: f32,
    cohesion: f32,
    alignment: f32,
    boid_count: i32,
    paused: bool
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            vision_distance: 100.0,
            size: 1.0,
            separation: 0.0,
            cohesion: 0.0,
            alignment: 0.0,
            boid_count: 100,
            paused: false
        }
    }
}

#[derive(Reflect, Clone, Component, Inspectable, Default)]
#[reflect(Component)] //Component has a transform
pub struct Direction(Vec2);




