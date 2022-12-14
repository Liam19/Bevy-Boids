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
    .register_type::<Boid>()

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
        .insert(Boid {
            //alignment: Vec2::new(dir_x, dir_y).normalize(),
            velocity: Vec2::new(dir_x, dir_y).normalize(),
            ..default()
        });
    }
}

fn flocking_system(
    mut query: Query<(&GlobalTransform, &mut Boid)>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(t1, mut boid1), (t2, boid2)]) = combinations.fetch_next() {
        if t1.translation().distance(t2.translation()) < settings.vision_distance {
            boid1.neighbour_count += 1.0;
            
            boid1.cohesion += t2.translation().truncate();
            boid1.alignment += boid2.velocity 
        }
    }
}

fn movement_system(
    mut objects: Query<(&mut Boid, &mut Transform, &GlobalTransform)>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    if !settings.paused {
        for (mut boid, mut transform, global_transform) in &mut objects {

            if boid.cohesion.length() * settings.cohesion != 0.0 {
                let tgt_vec = (boid.cohesion / boid.neighbour_count) - global_transform.translation().truncate();
                boid.velocity = boid.velocity.lerp(tgt_vec, time.delta_seconds() * settings.cohesion / 10.0)
            }
            
            if boid.alignment.length() * settings.alignment != 0.0 {
                boid.velocity = boid.velocity.lerp(boid.alignment.normalize(), time.delta_seconds() * settings.alignment / 10.0);
            }
            
            // dont touch
            transform.translation += boid.velocity.extend(0.0).normalize() * time.delta_seconds() * settings.move_speed;
            boid.reset();
        }
    }
}

fn wrap_borders_system(
    mut objects: Query<&mut Transform, With<Boid>>,
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

fn resize_system(
    mut objects: Query<&mut Transform, With<Boid>>,
    settings: Res<Settings>
) {
    for mut transform in &mut objects {
        transform.scale.x = settings.size;
        transform.scale.y = settings.size;
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
            alignment: 1.0,
            boid_count: 100,
            paused: false
        }
    }
}

#[derive(Reflect, Clone, Component, Inspectable)]
#[reflect(Component)] //Component has a transform
pub struct Boid {
    separation: Vec2,
    cohesion: Vec2,
    alignment: Vec2,
    velocity: Vec2,
    neighbour_count: f32,
}

impl Default for Boid {
    fn default() -> Self {
        Self {
            separation: Vec2::Y,
            cohesion: Vec2::Y,
            alignment: Vec2::Y,
            velocity: Vec2::Y,
            neighbour_count: 0.0,
        }
    }
}

impl Boid {
    fn reset(
        &mut self,
    ) {
        self.cohesion = Vec2::ZERO;
        self.separation = Vec2::ZERO;
        self.alignment = Vec2::ZERO;
        self.neighbour_count = 0.0;
    }
}


