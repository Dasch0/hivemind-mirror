mod debug;
mod setup;

use crate::{grid, world, AppState};
use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesPlugin;

pub const WORLD_DRAW_SCALE: f32 = 1.0;

/// Helpers to ping a tile in the world
#[derive(Component)]
pub struct Ping;

pub fn ping(commands: &mut Commands, p: IVec2) {
    commands
        .spawn()
        .insert(Ping)
        .insert(world::Position(p.as_vec2()))
        .insert(Temp(Timer::from_seconds(0.5, false)));
}

pub fn ping_long(commands: &mut Commands, p: IVec2) {
    commands
        .spawn()
        .insert(Ping)
        .insert(world::Position(p.as_vec2()))
        .insert(Temp(Timer::from_seconds(1.0, false)));
}

#[derive(Component)]
pub struct WorldSpriteOffset;

#[derive(Component)]
pub struct WorldSpriteNoOffset;

#[derive(Component)]
pub struct HealthBarSprite;

impl HealthBarSprite {
    pub const OFFSET: Vec3 = bevy::math::const_vec3!([0.0, 50.0, 5.0]);
}

/// Updates all sprites drawn to the world with offset (things centered at tile)
pub fn world_sprites_offset(mut query: Query<(&world::Position, &mut Transform), With<WorldSpriteOffset>>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = grid::world_to_iso(pos.0);
    }
}

/// Updates all sprites drawn to the world with offset
pub fn world_sprites_no_offset(mut query: Query<(&world::Position, &mut Transform), With<WorldSpriteNoOffset>>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = grid::world_to_iso_no_offset(pos.0);
    }
}

/// Updates all health bars based on resource value drawn to the world with offset (things centered at tile)
pub fn healthbars(map: Res<world::WorldMap>, mut query: Query<(&world::Position, &mut Transform), With<HealthBarSprite>>) {
    for (pos, mut transform) in query.iter_mut() {
            transform.scale = Vec3::new((map[pos.0].get_resource_quantity() * 100 / world::Colony::MAX) as f32 + 2.0, 8.0, 0.0);
    }
}

#[derive(Component)]
pub struct Delay(pub Timer);

#[derive(Component)]
pub struct Temp(pub Timer);

pub fn despawn_temp(
    mut commands: Commands,
    time_step: Res<world::TimeStep>,
    mut query: Query<(Entity, &mut Temp)>,
) {
    let time_step = time_step.into_inner();
    for (entity, mut clock) in query.iter_mut() {
        if clock.0.tick(time_step.into()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct Plugin {
    pub debug: bool,
}

pub struct Config {
    pub debug: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DebugDraw {
    On,
    Off,
}

pub fn debug_toggle(
    mut state: ResMut<State<DebugDraw>>,
    mut config: ResMut<Config>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::T) {
        config.debug = !config.debug;
        if config.debug {
            state.set(DebugDraw::On).unwrap();
        } else {
            state.set(DebugDraw::Off).unwrap();
        }
    }
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Playing)
                .with_system(setup::hivemind)
                .with_system(setup::colony)
                .with_system(setup::flower)
                .with_system(setup::tree)
                .with_system(setup::volcano)
                .with_system(setup::multivac)
                .with_system(setup::wire)
                .with_system(setup::outpost),
        )
        //.add_system_set(SystemSet::on_enter(CoreStage::Last).with_system(setup_particles))
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .after(world::Order::WorldUpdate)
                .with_system(setup::hivemind)
                .with_system(setup::colony)
                .with_system(setup::flower)
                .with_system(setup::tree)
                .with_system(setup::volcano)
                .with_system(setup::multivac)
                .with_system(setup::wire)
                .with_system(setup::outpost)
                .with_system(setup::ping)
                .with_system(world_sprites_offset)
                .with_system(world_sprites_no_offset)
                .with_system(healthbars)
                .with_system(despawn_temp),
        );
        app.add_plugin(DebugLinesPlugin::default())
            .add_state(if self.debug {
                DebugDraw::On
            } else {
                DebugDraw::Off
            })
            .insert_resource(Config { debug: self.debug })
            .add_system(debug_toggle)
            .add_startup_system_set_to_stage(
                StartupStage::PostStartup,
                SystemSet::new().with_system(debug::setup_fields),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::Playing), //.with_system(debug::sprites)
            )
            .add_system_set(
                SystemSet::on_update(DebugDraw::On)
                    .with_system(debug::vector_fields)
                    .with_system(debug::scalar_fields),
            );
    }
}
