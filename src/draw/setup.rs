/// Setup functions for drawing. These don't necessarily have to be run as startup systems. They
/// should be able to detect objects that havent been registered for drawing yet
use bevy::prelude::*;

use crate::{
    grid,
    hivemind::colony,
    multivac::{Dir, WireKind},
    texture::{TextureAtlases, TextureHandles},
    world::{self, Flag},
};

use super::{Delay, Ping, WorldSpriteNoOffset, WorldSpriteOffset, WORLD_DRAW_SCALE};

/// add data for any undrawn hivemind sprites
pub fn hivemind(
    mut commands: Commands,
    sprite_sheets: Res<TextureAtlases>,
    drones: Query<(Entity, &colony::Drone, &world::Position, &world::Flag), Without<Transform>>,
) {
    for (entity, _, pos, colony) in drones.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteSheetBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture_atlas: match *colony & (Flag::COLONY_C | Flag::COLONY_M | Flag::COLONY_Y) {
                    //@TODO do bee_all sheet :)
                    Flag::COLONY_ALL => sprite_sheets["bee_all"].clone(),
                    Flag::COLONY_C => sprite_sheets["bee_c"].clone(),
                    Flag::COLONY_M => sprite_sheets["bee_m"].clone(),
                    Flag::COLONY_Y => sprite_sheets["bee_y"].clone(),
                    _ => panic!("A drone was instantiated with invalid colony bit flags"),
                },
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },

                ..Default::default()
            })
            .insert(WorldSpriteNoOffset);
    }
}

/// Add data for any undrawn flower sprites
pub fn flower(
    mut commands: Commands,
    sprite_sheets: Res<TextureAtlases>,
    query: Query<(Entity, &world::Flower, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteSheetBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture_atlas: sprite_sheets["flower"].clone(),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(WorldSpriteOffset);
    }
}

/// Add data for any undrawn tree sprites
pub fn tree(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &world::Tree, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture: textures["tree"].clone(),
                ..Default::default()
            })
            .insert(WorldSpriteOffset);
    }
}

/// Add data for any undrawn volcano sprites
pub fn volcano(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &world::Volcano, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture: textures["volcano"].clone(),
                ..Default::default()
            })
            .insert(WorldSpriteOffset);
    }
}

/// Add data for any undrawn colony sprites
pub fn colony(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &world::Colony, &world::Flag, &world::Position), Without<Transform>>,
    map: Res<world::WorldMap>,
) {
    for (entity, _, flag, pos) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture: match *flag {
                    world::Flag::COLONY_C => textures["colony_c"].clone(),
                    world::Flag::COLONY_M => textures["colony_m"].clone(),
                    world::Flag::COLONY_Y => textures["colony_y"].clone(),
                    world::Flag::COLONY_ALL => textures["colony"].clone(),
                    _ => panic!(
                        "invalid flags {:?} for drawing colony at {}, {}",
                        flag, pos.0.x, pos.0.y
                    ),
                },
                ..Default::default()
            })
            .insert(WorldSpriteOffset)
            .with_children(|parent| {
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: match *flag {
                                world::Flag::COLONY_Y => Color::rgba(1.0, 0.74, 0.0, 0.8),
                                world::Flag::COLONY_C => Color::rgba(0.12, 0.86, 0.63, 0.8),
                                world::Flag::COLONY_M => Color::rgba(0.89, 0.24, 0.75, 0.8),
                                world::Flag::COLONY_ALL => Color::rgba(0.9, 0.9, 0.9, 0.8),
                                _ => panic!(
                                    "invalid flags {:?} for drawing colony at {}, {}",
                                    flag, pos.0.x, pos.0.y
                                ),
                            },
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: super::HealthBarSprite::OFFSET,
                            scale: Vec3::new(
                                0.5 * map[pos.0].get_resource_quantity() as f32,
                                10.0,
                                0.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(super::HealthBarSprite)
                    .insert(world::Position(pos.0));
            });

        println!("test");
    }
}

/// Add data for any undrawn multivac sprites
pub fn multivac(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &world::Multivac, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        println!("multivac!");
        commands.entity(entity).insert_bundle(SpriteBundle {
            transform: Transform {
                translation: grid::world_to_iso(pos.0),
                scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                ..Default::default()
            },
            texture: textures["multivac"].clone(),
            ..Default::default()
        });
    }
}

/// Add data for any undrawn multivac sprites
// TODO: Figure out if this should be a single wire sprite sheet, or multiple sprites assigned
// based on the direction faced
pub fn wire(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    time_step: Res<world::TimeStep>,
    mut query: Query<
        (
            Entity,
            &world::Wire,
            &WireKind,
            &world::Position,
            Option<&mut Delay>,
        ),
        Without<Transform>,
    >,
) {
    let time_step = time_step.into_inner();
    for (entity, _, kind, pos, delay) in query.iter_mut() {
        if let Some(mut delay) = delay {
            if delay.0.tick(time_step.into()).finished() {
                commands.entity(entity).insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: grid::world_to_iso(pos.0),
                        scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                        ..Default::default()
                    },
                    texture: match kind {
                        WireKind::Disconnect(Dir::North) => textures["wire-north"].clone(),
                        WireKind::Disconnect(Dir::East) => textures["wire-east"].clone(),
                        WireKind::Disconnect(Dir::South) => textures["wire-south"].clone(),
                        WireKind::Disconnect(Dir::West) => textures["wire-west"].clone(),
                        WireKind::Connect(Dir::North, Dir::South) => {
                            textures["wire-north-south"].clone()
                        }
                        WireKind::Connect(Dir::North, Dir::East) => {
                            textures["wire-north-east"].clone()
                        }
                        WireKind::Connect(Dir::North, Dir::West) => {
                            textures["wire-north-west"].clone()
                        }
                        WireKind::Connect(Dir::South, Dir::North) => {
                            textures["wire-north-south"].clone()
                        }
                        WireKind::Connect(Dir::South, Dir::East) => {
                            textures["wire-south-east"].clone()
                        }
                        WireKind::Connect(Dir::South, Dir::West) => {
                            textures["wire-south-west"].clone()
                        }
                        WireKind::Connect(Dir::East, Dir::North) => {
                            textures["wire-north-east"].clone()
                        }
                        WireKind::Connect(Dir::East, Dir::South) => {
                            textures["wire-south-east"].clone()
                        }
                        WireKind::Connect(Dir::East, Dir::West) => {
                            textures["wire-east-west"].clone()
                        }
                        WireKind::Connect(Dir::West, Dir::North) => {
                            textures["wire-north-west"].clone()
                        }
                        WireKind::Connect(Dir::West, Dir::East) => {
                            textures["wire-east-west"].clone()
                        }
                        WireKind::Connect(Dir::West, Dir::South) => {
                            textures["wire-south-west"].clone()
                        }
                        _ => textures["wire-all"].clone(),
                    },
                    ..Default::default()
                });
            }
        }
    }
}

/// Add data for any undrawn outpost sprites
// TODO: Figure out if this should be a single wire sprite sheet, or multiple sprites assigned
// based on the direction faced
pub fn outpost(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &world::Outpost, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        commands.entity(entity).insert_bundle(SpriteBundle {
            transform: Transform {
                translation: grid::world_to_iso(pos.0),
                scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                ..Default::default()
            },
            texture: textures["outpost"].clone(),
            ..Default::default()
        });
    }
}

/// add data for any undrawn ping sprites
pub fn ping(
    mut commands: Commands,
    textures: Res<TextureHandles>,
    query: Query<(Entity, &Ping, &world::Position), Without<Transform>>,
) {
    for (entity, _, pos) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: grid::world_to_iso(pos.0),
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.0),
                    ..Default::default()
                },
                texture: textures["ping"].clone(),
                ..Default::default()
            })
            .insert(WorldSpriteOffset);
    }
}
