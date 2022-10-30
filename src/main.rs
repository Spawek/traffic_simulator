use std::{f32::consts::PI};

use bevy::prelude::*;

#[derive(Component, Default)]
struct Car {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub angle: f32,
    pub wheel_angle: f32,
}

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Arrow;

#[derive(Component)]
struct KeyboardSteered;

// car_tiny.png
const CAR_WIDTH: f32 = 25.0;
const CAR_HEIGHT: f32 = 50.0;

const ARROW_LENGTH: f32 = CAR_HEIGHT * 0.8;
const ARROW_LAYER: f32 = 10.0;

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
        .insert(Car {
            angle: 45.0,
            wheel_angle: 10.0,
            speed: 2.0,
            ..Default::default()
        })
        .insert(Name("C1".to_string()))
        .insert(KeyboardSteered)
        .id();

    let c1_arrow = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, ARROW_LENGTH / 2.0, ARROW_LAYER),
                scale: Vec3::new(2.0, ARROW_LENGTH, 0.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            ..default()
        })
        .insert(Arrow)
        .id();

    commands.entity(c1).add_child(c1_arrow);

    // commands
    //     .spawn_bundle(SpriteBundle {
    //         texture: car_handle.clone(),
    //         transform: Transform::default(),
    //         ..default()
    //     })
    //     .insert(Car {
    //         x: 100.0,
    //         y: -200.0,
    //         ..Default::default()
    //     })
    //     .insert(Name("C2".to_string()));

    // commands
    //     .spawn_bundle(SpriteBundle {
    //         texture: car_handle.clone(),
    //         transform: Transform::default(),
    //         ..default()
    //     })
    //     .insert(Car {
    //         x: 400.0,
    //         y: 100.0,
    //         ..Default::default()
    //     })
    //     .insert(Name("C3".to_string()));

    commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(100.0, 2.0, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::RED,
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(2.0, 100.0, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::RED,
            ..default()
        },
        ..default()
    });
}

struct CarTransformUpdateTimer(Timer);
struct CarUpdateTimer(Timer);
struct GetKeyboardSignalsTimer(Timer);

struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CarUpdateTimer(Timer::from_seconds(1.0 / 60.0, true)))
        .insert_resource(CarTransformUpdateTimer(Timer::from_seconds(1.0 / 60.0, true)))
        .insert_resource(GetKeyboardSignalsTimer(Timer::from_seconds(1.0 / 60.0, true)))
            .add_startup_system(setup_camera)
            .add_startup_system(add_cars)
            .add_system(update_cars)
            .add_system(update_car_transforms)
            .add_system(get_keyboard_signals);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn update_cars(
    time: Res<Time>,
    mut timer: ResMut<CarUpdateTimer>,
    mut query: Query<&mut Car, (With<Car>, Without<Arrow>)>  // TODO: w/o arrow needed?
){
    const ROTATION_SPEED: f32 = 1.0;
    const MOVE_SPEED: f32 = 0.3;

    if timer.0.tick(time.delta()).just_finished() {
        for mut car in query.iter_mut() {
            car.x += car.speed * car.angle.to_radians().sin() * MOVE_SPEED;
            car.y += car.speed * car.angle.to_radians().cos() * MOVE_SPEED;
            car.angle += car.speed * car.wheel_angle.to_radians().sin() * ROTATION_SPEED;
            car.angle = car.angle % 360.0;
        }
    }
}

fn update_car_transforms(
    time: Res<Time>,
    mut timer: ResMut<CarTransformUpdateTimer>,
    mut query: Query<(&mut Transform, &Children, &Car), (With<Car>, Without<Arrow>)>,
    mut child_query: Query<&mut Transform, With<Arrow>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut transform, children, car) in query.iter_mut() {
            transform.rotation = Quat::from_rotation_z(-1.0 * car.angle.to_radians());
            transform.translation.x = car.x;
            transform.translation.y = car.y;

            // Adjust arrow - it should change only when the direction changes.
            for &child in children.iter() {
                let mut arrow = child_query.get_mut(child).unwrap();
                let rad = -1.0 * car.wheel_angle / 360.0 * (2.0 * PI);
                arrow.rotation = Quat::from_rotation_z(rad);
                arrow.translation = Vec3::new(
                    -1.0 * ARROW_LENGTH / 2.0 * rad.sin(),
                    ARROW_LENGTH / 2.0 * rad.cos(),
                    ARROW_LAYER,
                );
            }
        }
    }
}

fn get_keyboard_signals(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut timer: ResMut<GetKeyboardSignalsTimer>,
    mut query: Query<&mut Car, (With<Car>, With<KeyboardSteered>, Without<Arrow>)>  // TODO: w/o arrow needed?
) {
    const SPEED_INCREMENT: f32 = 0.1;
    const MIN_SPEED : f32 = 0.0;
    const MAX_SPEED : f32 = 5.0;

    const ANGLE_INCREMENT: f32 = 0.5;
    const MIN_ANGLE : f32 = -35.0;
    const MAX_ANGLE : f32 = 35.0;

    if timer.0.tick(time.delta()).just_finished() {
        for mut car in query.iter_mut() {
            if keys.pressed(KeyCode::W){
                car.speed += SPEED_INCREMENT;
            }
            if keys.pressed(KeyCode::S){
                car.speed -= SPEED_INCREMENT;
            }
            if keys.pressed(KeyCode::A){
                car.wheel_angle -= ANGLE_INCREMENT;
            }
            if keys.pressed(KeyCode::D){
                car.wheel_angle += ANGLE_INCREMENT;
            }
            car.speed = car.speed.max(MIN_SPEED).min(MAX_SPEED);
            car.wheel_angle = car.wheel_angle.max(MIN_ANGLE).min(MAX_ANGLE);
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
// make the angle length proportional to speed
// change angle steering to automatically going back to 0