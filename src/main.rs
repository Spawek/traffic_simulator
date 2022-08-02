use std::f32::consts::PI;

use bevy::prelude::*;

#[derive(Component)]
struct Car;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct CarDirection(f32);

// car_tiny.png
const CAR_WIDTH : f32 = 25.0;
const CAR_HEIGHT : f32 = 50.0;

const ARROW_LENGTH : f32 = CAR_HEIGHT * 0.8;
const ARROW_LAYER : f32 = 10.0;

fn add_cars(mut commands: Commands, asset_server: Res<AssetServer>) {
    let car_handle = asset_server.load("car_tiny.png");
    let c1 = commands
        .spawn_bundle(SpriteBundle {
            texture: car_handle.clone(),
            transform: Transform {
                translation: Vec3::from((0.0, 0.0, 0.0)),
                ..Default::default()
            },
            ..default()
        })
        .insert(Car)
        .insert(Name("C1".to_string())).id();

    let c1_arrow = commands
        .spawn_bundle(SpriteBundle{
            transform: Transform { translation: Vec3::new(0.0, ARROW_LENGTH / 2.0, ARROW_LAYER), scale: Vec3::new(2.0, ARROW_LENGTH, 0.0), ..default() },
            sprite: Sprite { color: Color::RED, ..default() },
            ..default()})
        .insert(CarDirection(10.0)).id();
        
    commands.entity(c1).add_child(c1_arrow);

    commands
        .spawn_bundle(SpriteBundle {
            texture: car_handle.clone(),
            transform: Transform {
                translation: Vec3::from((100.0, -200.0, 0.0)),
                ..Default::default()
            },
            ..default()
        })
        .insert(Car)
        .insert(Name("C2".to_string()));

    commands
        .spawn_bundle(SpriteBundle {
            texture: car_handle.clone(),
            transform: Transform {
                translation: Vec3::from((400.0, 100.0, 0.0)),
                ..Default::default()
            },
            ..default()
        })
        .insert(Car)
        .insert(Name("C3".to_string()));

    commands
    .spawn_bundle(SpriteBundle{
        transform: Transform { translation: Vec3::new(0.0, 0.0, 0.0), scale: Vec3::new(100.0, 2.0, 0.0), ..default() },
        sprite: Sprite { color: Color::RED, ..default() },
        ..default()
    });

    commands
    .spawn_bundle(SpriteBundle{
        transform: Transform { translation: Vec3::new(0.0, 0.0, 0.0), scale: Vec3::new(2.0, 100.0, 0.0), ..default() },
        sprite: Sprite { color: Color::RED, ..default() },
        ..default()
    });
}

struct CarUpdateTimer(Timer);

struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CarUpdateTimer(Timer::from_seconds(1.0 / 60.0, true)))
            .add_startup_system(setup_camera)
            .add_startup_system(add_cars)
            .add_system(update_car_positions);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn update_car_positions(
    time: Res<Time>,
    mut timer: ResMut<CarUpdateTimer>,
    mut query: Query<(&mut Transform, &Children), (With<Car>, Without<CarDirection>)>,
    mut child_query: Query<(&mut Transform, &mut CarDirection), With<CarDirection>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut transform, children) in query.iter_mut() {
            transform.rotation = Quat::from_rotation_z(time.seconds_since_startup() as f32);
            for &child in children.iter() {
                let (mut arrow, mut direction) = child_query.get_mut(child).unwrap();
                direction.0 += 0.2;
                let rad = -1.0 * direction.0 / 360.0  * (2.0 * PI);
                arrow.rotation = Quat::from_rotation_z(rad);
                arrow.translation = Vec3::new(-1.0 * ARROW_LENGTH / 2.0  * rad.sin(), ARROW_LENGTH / 2.0  * rad.cos(), ARROW_LAYER);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CarPlugin)
        .run();
}

// TODO: 
// make the car steerable from keyboard
