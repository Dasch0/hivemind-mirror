pub mod colony;
pub mod field;
pub mod field_systems;

pub use colony::Drone;

/// Hivemind AI implementation. Ported (with modifications) from johnBuffer's incredible [Ant Simulator project](https://github.com/johnBuffer/AntSimulator/blob/master/include/simulation/world/world_grid.hpp)
use bevy::prelude::*;
use bevy::tasks::prelude::*;

use crate::world;

/// Convenience type for world sized vector field
pub type VectorField = field::Vector<{ crate::WORLD_SIZE }, { crate::WORLD_SIZE }>;
/// Convenience types for world sized scalar fields
pub type ScalarField = field::Scalar<{ crate::WORLD_SIZE }, { crate::WORLD_SIZE }>;

pub struct GatherEvent(pub Vec2);
pub struct DepositEvent(pub Vec2);

/// udpate the world with gather info
pub fn gather(mut map: ResMut<world::WorldMap>, mut events: EventReader<GatherEvent> ) {
    for event in events.iter() {
        let resource = map[event.0].get_resource_quantity();
        if resource > 0 {
            map[event.0].set_resource_quantity(resource - 1);
        } else {
            map[event.0] &= !world::Flag::HIVE_FOOD;
        }
    }
}

/// update the world with deposit info
pub fn deposit(mut map: ResMut<world::WorldMap>, mut events: EventReader<DepositEvent> ) {
    for event in events.iter() {
        let resource = map[event.0].get_resource_quantity();
        if resource < world::Flag::MAX_RESOURCE_COUNT {
            map[event.0].set_resource_quantity(resource + 1);
        }
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(colony::Config::default())
            .add_event::<GatherEvent>()
            .add_event::<DepositEvent>()
            .add_startup_system_set_to_stage(
                StartupStage::Startup,
                SystemSet::new()
                    .label(world::Order::WorldInit)
                    .with_system(field_systems::setup.label(world::Order::WorldInit)),
            )
            .add_startup_system_set_to_stage(
                StartupStage::Startup,
                SystemSet::new()
                    .label(world::Order::EntityInit)
                    .after(world::Order::WorldInit)
                    .with_system(colony::setup),
            )
            .add_system_set(
                SystemSet::new()
                    .label(world::Order::WorldUpdate)
                    .with_system(update_all_scalar_fields)
                    .with_system(update_all_vector_fields)
                    .with_system(gather)
                    .with_system(deposit),
            )
            .add_system_set(
                SystemSet::new()
                    .label(world::Order::EntityUpdate)
                    .after(world::Order::WorldUpdate)
                    .with_system(colony::update_drones)
                    .with_system(colony::update_drone_sprites)
                    .with_system(colony::update_colony),
            )
            .add_system_set(
                SystemSet::new()
                    .label(world::Order::EntityFeedback)
                    .after(world::Order::WorldUpdate)
                    .with_system(field_systems::update_attractor)
                    .with_system(field_systems::update_repellent)
                    .with_system(field_systems::update_density)
                    .with_system(field_systems::update_world)
                    .with_system(colony::signal_drones),
            );
    }
}

/// Convenience system to update simulation of all scalar fields
pub fn update_all_scalar_fields(
    pool: Res<ComputeTaskPool>,
    time_step: Res<world::TimeStep>,
    mut fields: Query<&mut ScalarField>,
) {
    let real_number: f32 = time_step.into_inner().into();
    fields.par_for_each_mut(&pool, 1, |mut field| field.update(real_number));
}

/// Convenience system to update simulation of all vector fields
pub fn update_all_vector_fields(
    pool: Res<ComputeTaskPool>,
    time_step: Res<world::TimeStep>,
    mut fields: Query<&mut VectorField>,
) {
    let real_number: f32 = time_step.into_inner().into();
    fields.par_for_each_mut(&pool, 1, |mut field| field.update(real_number));
}
