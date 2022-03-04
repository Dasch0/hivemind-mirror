use crate::{
    hivemind::{colony::DroneState, Drone, ScalarField, VectorField},
    world::{self, WorldMap},
};
/// Specific field implementations used by the hivemind
use bevy::prelude::*;

#[derive(Component)]
pub struct Food;

#[derive(Component)]
pub struct Wall;

pub fn update_world(
    mut food_field: Query<&mut ScalarField, With<Food>>,
    mut wall_field: Query<&mut ScalarField, (With<Wall>, Without<Food>)>,
    mut attractor_field: Query<&mut VectorField, (With<Attractor>, Without<Food>, Without<Wall>)>,
    mut repellent_field: Query<&mut VectorField, (With<Repellent>, Without<Attractor>, Without<Food>, Without<Wall>)>,
    map: Res<WorldMap>,
) {
    let mut wall_field = wall_field.single_mut();
    let mut food_field = food_field.single_mut();
    let mut attractor_field = attractor_field.single_mut();
    let mut repellent_field = repellent_field.single_mut();

    for y in 0..map.data.len() {
        for x in 0..map.data.len() {
            if map[y][x].intersects(world::Flag::HIVE_FOOD) {
                food_field[y][x] += 100.0;
                wall_field[y][x] = 0.0;
            }
            if map[y][x].intersects(world::Flag::WALL) {
                wall_field[y][x] += 1.0;
                food_field[y][x] = 0.0;
                attractor_field[y][x] = Vec2::ZERO;
                repellent_field[y][x] = Vec2::ZERO;
            } if map[y][x].intersects(world::Flag::COLONY_ALL) {
                wall_field[y][x] = 0.0;
            }
        }
    }
}

#[derive(Component)]
pub struct Attractor;

pub fn update_attractor(
    mut field: Query<&mut VectorField, With<Attractor>>,
    drones: Query<(&world::Position, &Drone, &DroneState)>,
) {
    let mut field = field.single_mut();
    drones
        .iter()
        .filter(|(_, _, state)| **state == DroneState::ToHome)
        .for_each(|(pos, drone, _)| field[pos.0] += -drone.direction);
}

#[derive(Component)]
pub struct Repellent;

pub fn update_repellent(
    mut field: Query<&mut VectorField, With<Repellent>>,
    drones: Query<(&world::Position, &Drone, &DroneState)>,
) {
    let mut field = field.single_mut();
    drones
        .iter()
        .filter(|(_, _, state)| **state == DroneState::ToHomeNoFood)
        .for_each(|(pos, drone, _)| field[pos.0] -= -drone.direction);
}

#[derive(Component)]
pub struct Density;

pub fn update_density(
    mut field: Query<&mut ScalarField, With<Density>>,
    drones: Query<(&world::Position, &Drone)>,
) {
    let mut field = field.single_mut();
    drones.for_each(|(pos, _)| field[pos.0] += 0.1);
}

pub fn setup(mut commands: Commands) {
    // FIXME: remove debug addition of food to grid
    commands.spawn().insert(ScalarField::default()).insert(Food);
    commands
        .spawn()
        .insert(VectorField::default())
        .insert(Attractor);
    commands
        .spawn()
        .insert(VectorField::default())
        .insert(Repellent);
    commands
        .spawn()
        .insert(ScalarField::default())
        .insert(Density);
    commands
        .spawn()
        .insert(ScalarField::default_wall())
        .insert(Wall);
    info!("setup colony fields");
}
