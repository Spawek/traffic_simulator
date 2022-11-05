use std::f32::consts::PI;

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

const MAX_ARROW_LENGTH: f32 = CAR_HEIGHT * 0.8;
const ARROW_LAYER: f32 = 10.0;

const MAX_SPEED: f32 = 5.0;

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
                translation: Vec3::new(0.0, MAX_ARROW_LENGTH / 2.0, ARROW_LAYER),
                scale: Vec3::new(2.0, MAX_ARROW_LENGTH, 0.0),
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

struct CarUpdateTimer(Timer);
struct DetectCollisionsTimer(Timer);
struct CarTransformUpdateTimer(Timer);
struct GetKeyboardSignalsTimer(Timer);
struct ArrowUpdateTimer(Timer);

struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CarUpdateTimer(Timer::from_seconds(1.0 / 60.0, true)))
            .insert_resource(DetectCollisionsTimer(Timer::from_seconds(
                /*1.0 / 60.0*/ 1.0, true,
            ))) // TEMP CHANGED TO 1S
            .insert_resource(CarTransformUpdateTimer(Timer::from_seconds(
                1.0 / 60.0,
                true,
            )))
            .insert_resource(GetKeyboardSignalsTimer(Timer::from_seconds(
                1.0 / 60.0,
                true,
            )))
            .insert_resource(ArrowUpdateTimer(Timer::from_seconds(1.0 / 60.0, true)))
            .add_startup_system(setup_camera)
            .add_startup_system(add_cars)
            .add_system(update_cars)
            .add_system(detect_collisions)
            .add_system(update_car_transforms)
            .add_system(update_arrows)
            .add_system(get_keyboard_signals);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn update_cars(
    time: Res<Time>,
    mut timer: ResMut<CarUpdateTimer>,
    mut query: Query<&mut Car, (With<Car>, Without<Arrow>)>, // TODO: w/o arrow needed?
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    const ROTATION_SPEED: f32 = 1.0;
    const MOVE_SPEED: f32 = 0.3;

    for mut car in query.iter_mut() {
        car.x += car.speed * car.angle.to_radians().sin() * MOVE_SPEED;
        car.y += car.speed * car.angle.to_radians().cos() * MOVE_SPEED;
        car.angle += car.speed * car.wheel_angle.to_radians().sin() * ROTATION_SPEED;
        car.angle = car.angle % 360.0;
    }
}

#[derive(Clone, Copy, Debug)]
struct XY {
    x: f32,
    y: f32,
}

type Line = (XY, XY);

fn car_edges(car: &Car) -> [XY; 4] {
    // half of the diagonal 
    let d = ((CAR_WIDTH / 2.0).powi(2) + (CAR_HEIGHT / 2.0).powi(2)).sqrt();
    // angle to top right diagonal for an axis-alligned car
    let beta = ((CAR_WIDTH / 2.0) / (CAR_HEIGHT / 2.0)).atan().to_degrees();

    let angles = [
        (car.angle + beta).to_radians(),
        (180.0 - car.angle - beta).to_radians(),
        (180.0 + car.angle + beta).to_radians(),
        (car.angle - beta).to_radians(),
    ];
    
    angles
        .map(|angle| XY {
            x: car.x + d * angle.sin(),
            y: car.y + d * angle.cos(),
        })
}

fn car_borders(car: &Car) -> [Line; 4] {
    let edges = car_edges(car);

    [
        (edges[0], edges[1]),
        (edges[1], edges[2]),
        (edges[2], edges[3]),
        (edges[3], edges[0])
    ]
}

fn detect_collisions(
    time: Res<Time>,
    mut timer: ResMut<DetectCollisionsTimer>,
    query: Query<&Car>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let car_borders = query.iter().map(car_borders).collect::<Vec<_>>();
}

fn update_car_transforms(
    time: Res<Time>,
    mut timer: ResMut<CarTransformUpdateTimer>,
    mut query: Query<(&mut Transform, &Car), (With<Car>, Without<Arrow>)>,
    mut child_query: Query<&mut Transform, With<Arrow>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (mut transform, car) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(-1.0 * car.angle.to_radians());
        transform.translation.x = car.x;
        transform.translation.y = car.y;
    }
}

fn update_arrows(
    time: Res<Time>,
    mut timer: ResMut<ArrowUpdateTimer>,
    mut query: Query<(&Children, &Car), (With<Car>, Without<Arrow>)>,
    mut child_query: Query<&mut Transform, With<Arrow>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {}

    for (children, car) in query.iter_mut() {
        let arrow_length = car.speed / MAX_SPEED * MAX_ARROW_LENGTH;
        for &child in children.iter() {
            let mut arrow = child_query.get_mut(child).unwrap();
            let rad = -1.0 * car.wheel_angle / 360.0 * (2.0 * PI);
            arrow.rotation = Quat::from_rotation_z(rad);
            arrow.translation = Vec3::new(
                -1.0 * arrow_length / 2.0 * rad.sin(),
                arrow_length / 2.0 * rad.cos(),
                ARROW_LAYER,
            );
            arrow.scale.y = arrow_length;
        }
    }
}

fn get_keyboard_signals(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut timer: ResMut<GetKeyboardSignalsTimer>,
    mut query: Query<&mut Car, (With<Car>, With<KeyboardSteered>, Without<Arrow>)>, // TODO: w/o arrow needed?
) {
    const SPEED_INCREMENT: f32 = 0.1;
    const MIN_SPEED: f32 = 0.0;

    const ANGLE_INCREMENT: f32 = 0.8;
    const MIN_ANGLE: f32 = -35.0;
    const MAX_ANGLE: f32 = 35.0;

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut car in query.iter_mut() {
        if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
            car.speed += SPEED_INCREMENT;
        }
        if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
            car.speed -= SPEED_INCREMENT;
        }
        if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
            car.wheel_angle -= ANGLE_INCREMENT;
        }
        if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
            car.wheel_angle += ANGLE_INCREMENT;
        }
        car.speed = car.speed.max(MIN_SPEED).min(MAX_SPEED);
        car.wheel_angle = car.wheel_angle.max(MIN_ANGLE).min(MAX_ANGLE);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CarPlugin)
        .run();
}

// TODO:
// collision detection
//      IDEA: get 4 border lines for each car and if any lines collide then cars collide?
//            check each 4 with the previous lines and then add them to the set of lines to prevent car intersecting with each other
//            it's N^2, which should be fine for now
// change angle steering to automatically going back to 0


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_collisions() {
        let car1 = Car{
            ..Default::default()
        };
        let car2 = Car{
            x: CAR_WIDTH / 3.0,
            angle: 20.0,
            ..Default::default()
        };

        // TODO: detect collisions test
    }
}