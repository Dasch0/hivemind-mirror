use crate::texture::TextureHandles;

/// Debug draw routines
use crate::{
    grid,
    hivemind::{self, field_systems},
    world,
};
use bevy::prelude::*;

use bevy_prototype_debug_lines::DebugLines;

use super::{WorldSpriteOffset, WORLD_DRAW_SCALE};

#[derive(Component)]
pub struct DebugColor(Color);

/// Test draw all sprites
pub fn sprites(mut commands: Commands, textures: Res<TextureHandles>) {
    for (index, texture) in textures.values().enumerate() {
        let world_pos = Vec2::splat(index as f32);
        let iso_pos = grid::world_to_iso(world_pos);
        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: iso_pos,
                    scale: Vec3::new(WORLD_DRAW_SCALE, WORLD_DRAW_SCALE, 0.5),
                    ..Default::default()
                },
                texture: texture.clone(),
                ..Default::default()
            })
            .insert(world::Position(world_pos))
            .insert(WorldSpriteOffset);
    }
}

/// assign debug color to field lines
pub fn setup_fields(
    mut commands: Commands,
    food_field: Query<(Entity, &hivemind::ScalarField), With<field_systems::Food>>,
    density_field: Query<(Entity, &hivemind::ScalarField), With<field_systems::Density>>,
    repellent_field: Query<(Entity, &hivemind::VectorField), With<field_systems::Repellent>>,
    attractor_field: Query<(Entity, &hivemind::VectorField), With<field_systems::Attractor>>,
    wall_field: Query<(Entity, &hivemind::ScalarField), With<field_systems::Wall>>,
) {
    let food_field = food_field.single().0;
    let density_field = density_field.single().0;
    let repellent_field = repellent_field.single().0;
    let attractor_field = attractor_field.single().0;
    let wall_field = wall_field.single().0;

    commands.entity(food_field).insert(DebugColor(Color::RED));
    commands.entity(wall_field).insert(DebugColor(Color::BLACK));
    /*
    commands
        .entity(density_field)
        .insert(DebugColor(Color::BLUE));
    */
    commands
        .entity(repellent_field)
        .insert(DebugColor(Color::ORANGE));
    commands
        .entity(attractor_field)
        .insert(DebugColor(Color::GREEN));
}

/// test draw vector field lines
const VECTOR_SIZE: f32 = 0.5;
const CROSS_SIZE: f32 = 0.125;
pub fn vector_fields(
    map: Res<world::WorldMap>,
    mut lines: ResMut<DebugLines>,
    fields: Query<(&hivemind::VectorField, &DebugColor)>,
) {
    fields.for_each(|(field, color)| {
        for y in 0..map.w() {
            for x in 0..map.h() {
                let start = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let end = start + field[start].clamp(Vec2::ZERO, Vec2::splat(VECTOR_SIZE));
                // draw x at grid center with colored line for gradient
                let mag = field[start].length();
                let cross_x = Vec2::new(mag, 0.0).normalize_or_zero() * Vec2::splat(CROSS_SIZE);
                let cross_y = Vec2::new(0.0, mag).normalize_or_zero() * Vec2::splat(CROSS_SIZE);
                lines.line_colored(
                    grid::world_to_iso(start - cross_x) + Vec3::Z,
                    grid::world_to_iso(start + cross_x) + Vec3::Z,
                    0.0,
                    color.0,
                );
                lines.line_colored(
                    grid::world_to_iso(start - cross_y) + Vec3::Z,
                    grid::world_to_iso(start + cross_y) + Vec3::Z,
                    0.0,
                    color.0,
                );
                lines.line_colored(
                    grid::world_to_iso(start) + Vec3::Z,
                    grid::world_to_iso(end) + Vec3::Z,
                    0.0,
                    color.0,
                );
            }
        }
    });

    trace!("drew hivemind debug fields");
}

/// test draw scalar field lines
pub fn scalar_fields(
    map: Res<world::WorldMap>,
    mut lines: ResMut<DebugLines>,
    fields: Query<(&hivemind::ScalarField, &DebugColor)>,
) {
    fields.for_each(|(field, color)| {
        for y in 1..map.h() - 1 {
            for x in 1..map.w() - 1 {
                let start = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let end = start + field.grad(start).normalize_or_zero() * VECTOR_SIZE;
                // draw x at grid center with colored line for gradient
                let mag = field[y][x];
                let cross_x = Vec2::new(mag, 0.0).normalize_or_zero() * Vec2::splat(CROSS_SIZE);
                let cross_y = Vec2::new(0.0, mag).normalize_or_zero() * Vec2::splat(CROSS_SIZE);
                lines.line_colored(
                    grid::world_to_iso(start - cross_x) + Vec3::Z,
                    grid::world_to_iso(start + cross_x) + Vec3::Z,
                    0.0,
                    color.0,
                );
                lines.line_colored(
                    grid::world_to_iso(start - cross_y) + Vec3::Z,
                    grid::world_to_iso(start + cross_y) + Vec3::Z,
                    0.0,
                    color.0,
                );
                lines.line_colored(
                    grid::world_to_iso(start) + Vec3::Z,
                    grid::world_to_iso(end) + Vec3::Z,
                    0.0,
                    color.0,
                );
            }
        }
    });

    trace!("drew hivemind debug fields");
}
