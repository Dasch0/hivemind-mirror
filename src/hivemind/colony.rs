use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy::tasks::prelude::*;
use rand::prelude::*;

use super::{field_systems, DepositEvent, GatherEvent, ScalarField, VectorField};
use crate::{
    world::{self, Flag, WorldMap},
    game,
    story,
};

use rand::rngs::SmallRng;

const TEST_HIVE_LOCATION: (f32, f32) = (
    (1 * crate::WORLD_SIZE / 4) as f32,
    (1 * crate::WORLD_SIZE / 4) as f32,
);
const STAGING_HIVE_LOCATION: (f32, f32) = (
    (2 * crate::WORLD_SIZE / 4) as f32,
    (1 * crate::WORLD_SIZE / 4) as f32,
);
const PROD_HIVE_LOCATION: (f32, f32) = (
    (1 * crate::WORLD_SIZE / 4) as f32,
    (2 * crate::WORLD_SIZE / 4) as f32,
);

#[derive(Debug)]
pub struct Config {
    pub starting: usize,
    pub max: usize,
    pub spawn_rate: f32,
    pub drone_cost: u32,
    pub location_y: Vec2,
    pub location_m: Vec2,
    pub location_c: Vec2,
    pub color: Color,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub chaos: f32,
    pub explore_threshold: f32,
}

impl Config {
    pub fn default() -> Self {
        Self {
            starting: 1,
            max: 10000,
            spawn_rate: 1.0,
            drone_cost: 9,
            location_y: Vec2::new(TEST_HIVE_LOCATION.0, TEST_HIVE_LOCATION.1),
            location_c: Vec2::new(STAGING_HIVE_LOCATION.0, STAGING_HIVE_LOCATION.1),
            location_m: Vec2::new(PROD_HIVE_LOCATION.0, PROD_HIVE_LOCATION.1),
            color: Color::rgb(1.0, 1.0, 1.0),
            move_speed: 1.0,
            turn_speed: 0.15,
            chaos: 1.0,
            explore_threshold: 0.1,
        }
    }
}

#[derive(Component)]
pub struct Base {
    pub location: Vec2,
    pub size: f32,
    pub food: f32,
}

#[derive(Component)]
pub struct Drone {
    pub direction: Vec2, // unit vector
    pub autonomy: bool,
}

#[derive(Component)]
pub struct Colonist {
    pub home: Vec2,
}

// colony colors, used to easily find and despawn the drones when game over occurs
#[derive(Component)]
pub struct C;
#[derive(Component)]
pub struct Y;
#[derive(Component)]
pub struct M;

impl Drone {
    pub fn new() -> Self {
        Self {
            direction: Vec2::ZERO,
            autonomy: false,
        }
    }

    pub fn should_face_left(&self) -> bool {
        // https://stackoverflow.com/questions/6247153/angle-from-2d-unit-vector/6247163#6247163
        // :prayge:
        let radians = (-self.direction.y).atan2(self.direction.x);

        radians >= (3. * FRAC_PI_4) || radians <= -FRAC_PI_4
    }
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DroneState {
    ToHome,
    ToHomeNoFood,
    ToFood,
    Exploring,
    Gathering,
    Depositing,
    Resting,
    Dead,
}

#[derive(Component)]
pub struct ColonyClock(pub Timer);

/// Top level colony setup fn
pub fn setup(mut commands: Commands, mut map: ResMut<world::WorldMap>, config: Res<Config>) {
    debug!("setting up colonies with config: {:?}", config);
    // NOTE: unrolled loop to be able to put specific marker structs into colony and colonist
    // componenets
    let colony_flag = Flag::COLONY_Y;
    let pos = match colony_flag {
        Flag::COLONY_Y => config.location_y,
        Flag::COLONY_M => config.location_m,
        Flag::COLONY_C => config.location_c,
        _ => unreachable!(),
    };

    commands
        .spawn()
        .insert(world::Colony)
        .insert(ColonyClock(Timer::from_seconds(config.spawn_rate, true)))
        .insert(world::Position(pos))
        .insert(colony_flag.clone())
        .insert(M);
    map[pos] |= colony_flag;
    map[pos].set_resource_quantity(1000);
    for _ in 0..config.starting {
        commands
            .spawn()
            .insert(Drone::new())
            .insert(DroneState::Exploring)
            .insert(world::Position(pos))
            .insert(Colonist {
                home: pos + Vec2::new(0.5, 0.5),
            })
            .insert(colony_flag.clone())
            .insert(Y);
        }

    let colony_flag = Flag::COLONY_M;
    let pos = match colony_flag {
        Flag::COLONY_Y => config.location_y,
        Flag::COLONY_M => config.location_m,
        Flag::COLONY_C => config.location_c,
        _ => unreachable!(),
    };

    commands
        .spawn()
        .insert(world::Colony)
        .insert(ColonyClock(Timer::from_seconds(config.spawn_rate, true)))
        .insert(world::Position(pos))
        .insert(colony_flag.clone())
        .insert(M);
    map[pos] |= colony_flag;
    map[pos].set_resource_quantity(1000);
    for _ in 0..config.starting {
        commands
            .spawn()
            .insert(Drone::new())
            .insert(DroneState::Exploring)
            .insert(world::Position(pos))
            .insert(Colonist {
                home: pos + Vec2::new(0.5, 0.5),
            })
            .insert(colony_flag.clone())
            .insert(M);
        }

    let colony_flag = Flag::COLONY_C; 
    let pos = match colony_flag {
        Flag::COLONY_Y => config.location_y,
        Flag::COLONY_M => config.location_m,
        Flag::COLONY_C => config.location_c,
        _ => unreachable!(),
    };

    commands
        .spawn()
        .insert(world::Colony)
        .insert(ColonyClock(Timer::from_seconds(config.spawn_rate, true)))
        .insert(world::Position(pos))
        .insert(colony_flag.clone())
        .insert(C);
    map[pos] |= colony_flag;
    map[pos].set_resource_quantity(1000);
    for _ in 0..config.starting {
        commands
            .spawn()
            .insert(Drone::new())
            .insert(DroneState::Exploring)
            .insert(world::Position(pos))
            .insert(Colonist {
                home: pos + Vec2::new(0.5, 0.5),
            })
            .insert(colony_flag.clone()).insert(C);
        }

    info!("setup colonies");
}

/// colony tries to spawn a new drone, if it runs out of resources it dies
pub fn update_colony(
    mut commands: Commands,
    mut map: ResMut<world::WorldMap>,
    time_step: Res<world::TimeStep>,
    config: Res<Config>,
    cheat: Res<world::Cheat>,
    asset_server: Res<AssetServer>,
    mut game_timer: ResMut<game::GameTimer>,
    mut query: Query<(Entity, &mut ColonyClock, &world::Position, &world::Flag)>,
) {
    let time_step = time_step.into_inner();
    for (entity, mut clock, pos, flag) in query.iter_mut() {
        if clock.0.tick(time_step.into()).just_finished() {
            let resource = map[pos.0].get_resource_quantity();
            if resource > 0 {
                map[pos.0]
                    .set_resource_quantity(resource - std::cmp::min(resource, config.drone_cost));
            } else {
                if !cheat.0{
                // first, kill the colony, all bees will get stuck in 'ToHome'
                map[pos.0] = Flag::EMPTY;
                commands.entity(entity).despawn_recursive();
                // second, stop the clock
                game_timer.0.set_repeating(false);
                // third, send game over message
                info!("game over!");
                }

            commands
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(110.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    // Use the `Text::with_section` constructor
                    text: Text::with_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "GAME OVER!",
                        TextStyle {
                            font: asset_server.load("fonts/monogram.ttf"),
                            font_size: 100.0,
                            color: Color::WHITE,
                        },
                        // Note: You can use `Default::default()` in place of the `TextAlignment`
                        TextAlignment {
                            horizontal: HorizontalAlign::Right,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                });
            }

            commands
                .spawn()
                .insert(Drone::new())
                .insert(DroneState::Exploring)
                .insert(world::Position(pos.0))
                .insert(Colonist {
                    home: pos.0 + Vec2::new(0.5, 0.5),
                })
                .insert(flag.clone());
        }
    }
}

// signal as a separate system to minimize mutable borrows
pub fn signal_drones(
    drones: Query<(&DroneState, &world::Position)>,
    mut gather_event: EventWriter<GatherEvent>,
    mut deposit_event: EventWriter<DepositEvent>,
) {
    for (state, pos) in drones.iter() {
        match *state {
            DroneState::Gathering => gather_event.send(GatherEvent(pos.0)),
            DroneState::Depositing => deposit_event.send(DepositEvent(pos.0)),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_drones(
    pool: Res<ComputeTaskPool>,
    time_step: Res<world::TimeStep>,
    map: Res<WorldMap>,
    config: Res<Config>,
    food_field: Query<&ScalarField, With<field_systems::Food>>,
    wall_field: Query<&ScalarField, With<field_systems::Wall>>,
    attractor_field: Query<&VectorField, With<field_systems::Attractor>>,
    repellent_field: Query<&VectorField, With<field_systems::Repellent>>,
    density_field: Query<&ScalarField, With<field_systems::Density>>,
    mut drones: Query<(
        &mut Drone,
        &mut DroneState,
        &mut world::Position,
        &Flag,
        &Colonist,
    )>,
) {
    let time_step = time_step.into_inner();
    let food_f = food_field.single();
    let wall_f = wall_field.single();
    let density_f = density_field.single();
    let attractor_f = attractor_field.single();
    let repellent_f = repellent_field.single();

    drones.par_for_each_mut(
        &pool,
        32,
        |(mut drone, mut state, mut pos, colony, colonist)| {
            let colony = *colony;
            //let (entity, mut drone) = q;
            let mut rng = SmallRng::from_entropy();

            let cell = match map.get_vec2(pos.0) {
                Some(f) => f,
                None => Flag::EMPTY,
            };

            // stuck-in-wall bounce
            if cell.intersects(Flag::WALL) {
                let bounce_direction = -drone.direction;

                pos.0 += bounce_direction * *time_step;
                let cell = map[pos.0]; // current cell
                if cell.intersects(Flag::WALL) {
                    pos.0 += bounce_direction * *time_step * 10.0;
                    // early return, try again next frame
                    return
                }
            }

            // state change
            *state = match *state {
                DroneState::ToHome if cell.intersects(Flag::COLONY_ALL) => DroneState::Depositing,
                DroneState::ToHome if !cell.intersects(Flag::COLONY_ALL) => DroneState::ToHome,
                DroneState::ToHomeNoFood if cell.intersects(Flag::COLONY_ALL) => {
                    DroneState::Resting
                }
                DroneState::ToHomeNoFood if !cell.intersects(colony) => DroneState::ToHomeNoFood,
                DroneState::ToFood if cell.intersects(Flag::HIVE_FOOD) => DroneState::Gathering,
                DroneState::ToFood if !cell.intersects(Flag::HIVE_FOOD) && drone.autonomy => {
                    DroneState::Exploring
                }
                DroneState::Exploring if !drone.autonomy => DroneState::ToFood,
                DroneState::Exploring if drone.autonomy => DroneState::Exploring,
                DroneState::Gathering if cell.intersects(Flag::HIVE_FOOD) => DroneState::ToHome,
                DroneState::Gathering if !cell.intersects(Flag::HIVE_FOOD) => {
                    DroneState::ToHomeNoFood
                } // food source ran out
                DroneState::Depositing => DroneState::ToFood,
                DroneState::Resting => DroneState::ToFood,
                DroneState::Dead => DroneState::Dead, // death is a special state, only entered by calling kill
                _ => *state,
            };

            // pick which signals (if any) the drone cares about
            let local_density = density_f.grad(pos.0) * 0.05;
            let food_gradient = food_f.grad(pos.0);
            let signal = match *state {
                DroneState::ToHome => Some((colonist.home - pos.0 - local_density) * 5.0),
                DroneState::ToHomeNoFood => Some((colonist.home - pos.0 - local_density) * 5.0),
                DroneState::ToFood => {
                    Some(food_gradient + attractor_f[pos.0] - repellent_f[pos.0] - local_density)
                }
                DroneState::Exploring => Some(food_gradient - local_density),
                DroneState::Gathering => None,
                DroneState::Depositing => None,
                DroneState::Resting => None,
                DroneState::Dead => None,
            };

            // early exit if no signal
            if let Some(mut signal) = signal {
                // Determine the drones current autonomy based on signals they care about
                // non signals translate to no autonomy
                drone.autonomy = match *state {
                    DroneState::ToFood => {
                        if signal.length() < config.explore_threshold {
                            true
                        } else {
                            false
                        }
                    }
                    DroneState::Exploring => {
                        if food_gradient.length() > config.explore_threshold
                            || attractor_f[pos.0].length() > config.explore_threshold
                        {
                            false
                        } else {
                            true
                        }
                    }
                    _ => false,
                };

                // Determine where the drone should go next based on signals
                signal.x += rng.gen_range(-config.chaos..config.chaos);
                signal.y += rng.gen_range(-config.chaos..config.chaos);
                signal = signal.normalize_or_zero();

                let candidate_direction = drone
                    .direction
                    .lerp(signal, config.turn_speed)
                    .normalize_or_zero()
                    * config.move_speed;

                // look ahead to consider walls
                let candidate_pos = pos.0 + (*time_step * candidate_direction);
                // FIXME: lot of unneccessary computation here
                drone.direction = candidate_direction
                    .lerp(-0.5 * wall_f.grad(candidate_pos), config.turn_speed)
                    .normalize_or_zero()
                    * config.move_speed;

                let new_pos = pos.0 + (*time_step * drone.direction);

                // stop drone in its tracks if its at the edge of a wall, that way it can cleanly
                // 'bounce' off
                pos.0 = match map.get_vec2(new_pos) {
                    Some(flag) if flag.intersects(Flag::WALL) => {
                        // if you hit a wall, turn around
                        drone.direction *= -1.0;
                        pos.0 + (*time_step * drone.direction)
                    }
                    Some(flag) if !flag.intersects(Flag::WALL) => new_pos,
                    None => pos.0,
                    _ => pos.0, // should be impossible to reach
                }
            } else {
            }
        },
    );

    trace!("update drones");
}

pub fn update_drone_sprites(
    mut query: Query<(&Drone, &DroneState, &mut TextureAtlasSprite)>,
) {
    query.for_each_mut(|(drone, state, mut sprite)| {
        let mut idx = 0;
        if drone.should_face_left() {
            idx += 1
        }

        if *state == DroneState::ToHome {
            idx += 2;
        }

        if idx != sprite.index {
            sprite.index = idx;
        }
    })
}
