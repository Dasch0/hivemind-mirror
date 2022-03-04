use crate::{game, prelude::*};
/// global Information about the game world accessed by most modules
use bevy::prelude::*;
use bitflags::bitflags;

use std::fs::File;
use std::io::prelude::*;

pub const APOCALYPSE_COUNTDOWN: f32 = 200.0;

pub type WorldMap = Map<{ crate::WORLD_SIZE }, { crate::WORLD_SIZE }>;

pub use nanoserde::{DeRon, SerRon};

/// World time step value
// speed, delta_time
#[derive(Clone, Copy)]
pub struct TimeStep(pub f32, pub f32);

impl TimeStep {
    pub const STOP: Self = TimeStep(0.00, 0.);
    pub const PLAY: Self = TimeStep(1.0, 0.);
    pub const FAST: Self = TimeStep(2.0, 0.);
    pub const FASTER: Self = TimeStep(4.0, 0.);

    pub fn set_from(&mut self, other: &Self) {
        self.0 = other.0;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.0 = speed;
    }

    pub fn set_delta_time(&mut self, dt: f32) {
        self.1 = dt;
    }

    pub fn is_paused(&self) -> bool {
        self.0 == 0.0
    }

    pub fn toggle(&mut self) -> usize {
        if self.is_paused() {
            self.0 = Self::PLAY.0;
            3
        } else {
            self.0 = Self::STOP.0;
            1
        }
    }
}

impl From<TimeStep> for f32 {
    fn from(ts: TimeStep) -> Self {
        ts.0 * ts.1
    }
}

impl From<&TimeStep> for f32 {
    fn from(ts: &TimeStep) -> Self {
        ts.0 * ts.1
    }
}

impl From<TimeStep> for bevy::utils::Duration {
    fn from(ts: TimeStep) -> Self {
        std::time::Duration::from_secs_f32(ts.0 * ts.1)
    }
}

impl From<&TimeStep> for bevy::utils::Duration {
    fn from(ts: &TimeStep) -> Self {
        std::time::Duration::from_secs_f32(ts.0 * ts.1)
    }
}

impl std::ops::Mul<f32> for TimeStep {
    type Output = f32;
    fn mul(self, rhs: f32) -> Self::Output {
        self.0 * self.1 * rhs
    }
}

impl std::ops::Mul<TimeStep> for f32 {
    type Output = f32;
    fn mul(self, rhs: TimeStep) -> Self::Output {
        self * rhs.1 * rhs.0
    }
}

impl std::ops::Mul<Vec2> for TimeStep {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.0 * self.1 * rhs
    }
}

impl std::ops::Mul<TimeStep> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: TimeStep) -> Self::Output {
        self * rhs.1 * rhs.0
    }
}

/// World position component for an entity
#[derive(Component, Clone, Copy)]
pub struct Position(pub Vec2);

/// system labels following the order of operations in the hivemind simulation. Use to order
/// execution of external systems around hivemind
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum Order {
    WorldInit,
    EntityInit,
    WorldUpdate,
    EntityUpdate,
    EntityFeedback,
}

/// Contains all states about a given world cell that need global access as bitflags
///
// NOTE: If we end up with more than 16 states, keep bumping integer size in Ucell, don't forget to
// prepend the correct number of 0's to keep it all aligned and readable
pub type FlagType = u32;
bitflags! {
    #[derive(Component)]
    pub struct Flag: FlagType {
        const EMPTY             = 0b00000000000000000000000000000000;
        const FLOWER            = 0b00000000000000000000000000000001;
        const TREE              = 0b00000000000000000000000000000010;
        const VOLCANO           = 0b00000000000000000000000000000100;
        const COLONY_C          = 0b00000000000000000000000000001000;
        const COLONY_M          = 0b00000000000000000000000000010000;
        const COLONY_Y          = 0b00000000000000000000000000100000;
        const MULTIVAC          = 0b00000000000000000000000001000000;
        const WIRE              = 0b00000000000000000000000010000000;
        const OUTPOST           = 0b00000000000000000000000100000000;
        const CONNECTED         = 0b00000000000000000000001000000000;

        // helpers for valid sets of flags
        const HIVE_FOOD         = 0b00000000000000000000000000000001;
        const MULTIVAC_FOOD     = 0b00000000000000000000000000000111;
        const WALL              = 0b00000000000000000000000000000110; // Volcanos and trees block pathing
        const COLONY_ALL        = 0b00000000000000000000000000111000; // Colonies can combine
        const RESOURCE_QUANTITY = 0b11111111111111111111110000000000; // upper bits store generic resource quantity.
        const KIND_MASK         = !Self::RESOURCE_QUANTITY.bits;
                                                      // Resource interpretation depends on flags
    }
}

impl Flag {
    pub const QUANTITY_SHIFT: u32 = Self::RESOURCE_QUANTITY.bits.trailing_zeros();
    pub const MAX_RESOURCE_COUNT: u32 = 2u32.pow(Self::RESOURCE_QUANTITY.bits.leading_ones()) - 1;

    pub fn get_resource_quantity(&self) -> u32 {
        let bits = self.bits & Self::RESOURCE_QUANTITY.bits;
        bits >> Self::QUANTITY_SHIFT
    }

    pub fn set_resource_quantity(&mut self, count: u32) {
        // clear quantity bits
        self.bits &= Self::KIND_MASK.bits;

        self.bits |= count << Self::QUANTITY_SHIFT;
    }
}

#[derive(Component)]
pub struct Flower;

impl Flower {
    pub const MAX: u32 = 1000;
}

pub fn animate_despawn_flower(
    mut commands: Commands,
    map: Res<WorldMap>,
    mut query: Query<(Entity, &mut TextureAtlasSprite, &Position), With<Flower>>,
) {
    for (entity, mut sprite, pos) in query.iter_mut() {
        let idx = std::cmp::min(
            (Flower::MAX - map[pos.0].get_resource_quantity()) * 16 / Flower::MAX,
            16,
        );
        sprite.index = idx as usize;
        if !map[pos.0].intersects(Flag::FLOWER) {
            commands.entity(entity).despawn()
        }
    }
}

#[derive(Component)]
pub struct Tree;

impl Tree {
    pub const MAX: u32 = 1000;
}

pub fn despawn_tree(
    mut commands: Commands,
    map: Res<WorldMap>,
    query: Query<(Entity, &Position), With<Tree>>,
) {
    for (entity, pos) in query.iter() {
        if !map[pos.0].intersects(Flag::TREE) {
            commands.entity(entity).despawn()
        }
    }
}

#[derive(Component)]
pub struct Volcano;

pub fn despawn_volcano(
    mut commands: Commands,
    map: Res<WorldMap>,
    query: Query<(Entity, &Position), With<Volcano>>,
) {
    for (entity, pos) in query.iter() {
        if !map[pos.0].intersects(Flag::VOLCANO) {
            commands.entity(entity).despawn()
        }
    }
}

#[derive(Component)]
pub struct Colony;

impl Colony {
    pub const MAX: u32 = 1000;
}

#[derive(Component)]
pub struct Multivac;

#[derive(Component)]
pub struct Wire;

#[derive(Component)]
pub struct Outpost;

/// saveable world map
#[derive(SerRon, DeRon)]
pub struct SaveMap {
    data: Vec<FlagType>,
    width: usize,
    height: usize,
}

impl SaveMap {
    pub fn from_map<const W: usize, const H: usize>(map: &Map<W, H>) -> Self {
        let mut data: Vec<FlagType> = Vec::with_capacity(W * H);
        for y in 0..H {
            for x in 0..W {
                data.push(map[y][x].bits());
            }
        }

        Self {
            data,
            width: W,
            height: H,
        }
    }

    // TODO: remove assets hardcoded path
    pub fn load() -> Self {
        let bytes = include_str!("../assets/world.map");
        let save_map: SaveMap =
            DeRon::deserialize_ron(bytes).expect("ERROR: failed to load saved world map data");
        save_map
    }

    // TODO: remove assets hardcoded path
    pub fn save(&self) {
        let bytes = SerRon::serialize_ron(self);
        let mut file = File::create("assets/world.map")
            .expect("ERROR: failed to create save file for world map data");
        file.write_all(bytes.as_bytes())
            .expect("ERROR: failed to write to save file");
    }

    pub fn into_map<const W: usize, const H: usize>(&self) -> Map<W, H> {
        let mut map: Map<W, H> = Map::new();
        for y in 0..H {
            for x in 0..W {
                map[y][x] = Flag::from_bits(self.data[x + y * H])
                    .expect("failed to convert serialized data to world map");
            }
        }

        map
    }
}

// save the current world_map
pub fn save_system(map: &WorldMap) {
    let save_map = SaveMap::from_map(map);
    save_map.save();
}

// Implementation note - thoughts on world access pattern and planning:
// 0. (input) updates map with new stuff based on user input
// 1. (field) find local characters based on position
// 2. (hivemind) find correct field at location
// 3. (multivac)
//  a. obtain entire map of traversable, resource, and non-traversable cells
//  b. Maintains a graph of base locations and connecting grid cells to draw paths
//  c. Notified or queries and rebuilds graph each time map changes
// 4. (field) find field values at neighboring cells, update fields
// 5. (rendering) reads and obtain the information about each cell
//
// Based on this, grid should be global resource, updated once at the beginning of each frame based
// on input, then just repeatedly read by all other major systems
//
// To solve the locality issue, lets just use array indexing, resources are stored in a box on the heap
// anyway
pub struct Map<const W: usize, const H: usize> {
    pub data: [[Flag; W]; H],
}

impl<const W: usize, const H: usize> Map<W, H> {
    pub const WIDTH: usize = W;
    pub const HEIGHT: usize = H;

    pub fn new() -> Self {
        Self {
            data: [[Flag::EMPTY; W]; H],
        }
    }

    pub const fn w(&self) -> usize {
        W
    }

    pub const fn h(&self) -> usize {
        H
    }

    /// get with bounds check
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<Flag> {
        if x as usize <= W && y as usize <= H {
            Some(self[x][y])
        } else {
            None
        }
    }

    /// get with bounds check
    #[inline]
    pub fn get_vec2(&self, checked: Vec2) -> Option<Flag> {
        if checked.x > 0.0 && checked.y > 0.0 && checked.x as usize <= W && checked.y as usize <= H
        {
            Some(self[checked])
        } else {
            None
        }
    }

    /// get with bounds check
    #[inline]
    pub fn get_uvec2(&self, checked: UVec2) -> Option<Flag> {
        if checked.x as usize <= W && checked.y as usize <= H {
            Some(self[checked])
        } else {
            None
        }
    }

    /// get with bounds check
    #[inline]
    pub fn get_ivec2(&self, checked: IVec2) -> Option<Flag> {
        if checked.x > 0
            && checked.y > 0
            && ((checked.x as usize) < W)
            && ((checked.y as usize) < H)
        {
            Some(self[checked])
        } else {
            None
        }
    }

    pub fn initialize_map(mut commands: Commands, mut map: ResMut<WorldMap>) {
        let save_map = SaveMap::load();
        *map = save_map.into_map();
        for y in 0..save_map.height {
            for x in 0..save_map.width {
                if map[y][x].intersects(Flag::TREE) {
                    map[y][x].set_resource_quantity(Tree::MAX);
                    commands
                        .spawn()
                        .insert(Tree)
                        .insert(Position(Vec2::new(x as f32, y as f32)));
                }
                if map[y][x].intersects(Flag::FLOWER) {
                    map[y][x].set_resource_quantity(Flower::MAX);
                    commands
                        .spawn()
                        .insert(Flower)
                        .insert(Position(Vec2::new(x as f32, y as f32)));
                }
            }
        }
        let xs = [0, W - 1];
        let ys = [0, H - 1];
        for x in xs {
            for y in 1..H - 1 {
                if map[y][x].is_empty() {
                    map[y][x] = Flag::TREE;
                    map[y][x].set_resource_quantity(Tree::MAX);
                    commands
                        .spawn()
                        .insert(Tree)
                        .insert(Position(Vec2::new(x as f32, y as f32)));
                }
            }
        }
        for y in ys {
            for x in 1..W - 1 {
                if map[y][x].is_empty() {
                    map[y][x] = Flag::TREE;
                    map[y][x].set_resource_quantity(Tree::MAX);
                    commands
                        .spawn()
                        .insert(Tree)
                        .insert(Position(Vec2::new(x as f32, y as f32)));
                }
            }
        }

        for y in ys {
            for x in xs {
                if map[y][x].is_empty() {
                    map[y][x] = Flag::TREE;
                    map[y][x].set_resource_quantity(Tree::MAX);
                    commands
                        .spawn()
                        .insert(Tree)
                        .insert(Position(Vec2::new(x as f32, y as f32)));
                }
            }
        }
    }
}

impl<const W: usize, const H: usize> std::ops::Index<usize> for Map<W, H> {
    type Output = [Flag];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<Vec2> for Map<W, H> {
    type Output = Flag;

    fn index(&self, index: Vec2) -> &Self::Output {
        &self.data[index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<usize> for Map<W, H> {
    fn index_mut(&mut self, index: usize) -> &mut [Flag] {
        &mut self.data[index]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<Vec2> for Map<W, H> {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        &mut self.data[index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<UVec2> for Map<W, H> {
    type Output = Flag;

    fn index(&self, index: UVec2) -> &Self::Output {
        &self.data[index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<UVec2> for Map<W, H> {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        &mut self.data[index.y as usize][index.x as usize]
    }
}

pub fn update_timestep(mut timestep: ResMut<TimeStep>, time: Res<Time>) {
    timestep.set_delta_time(time.delta_seconds());
}

impl<const W: usize, const H: usize> std::ops::Index<IVec2> for Map<W, H> {
    type Output = Flag;

    fn index(&self, index: IVec2) -> &Self::Output {
        &self.data[index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<IVec2> for Map<W, H> {
    fn index_mut(&mut self, index: IVec2) -> &mut Self::Output {
        &mut self.data[index.y as usize][index.x as usize]
    }
}

pub struct Cheat(pub bool);

pub fn handle_event(
    mut commands: Commands,
    mut world_clicks: EventReader<WorldClickEvent>,
    key_input: Res<Input<KeyCode>>,
    map: ResMut<WorldMap>,
    mut cheat: ResMut<Cheat>,
    game_state: Res<crate::game::GameState>,
    mut game_writer: EventWriter<GameEvent>,
    _query: Query<(Entity, &Position)>,
) {
    let map = map.into_inner();
    for click in world_clicks.iter() {
        debug!("world click at {}, {} !", click.pos.x, click.pos.y);
        let click_pos = click.pos.floor();
        match click.btn {
            MouseButton::Left => {
                if game_state.flower_ammo > 0 && map[click_pos].is_empty() {
                    if !cheat.0 {
                        game_writer.send(GameEvent::Flower(ByteOp::Decrement));
                    }
                    commands.spawn().insert(Flower).insert(Position(click_pos));
                    map[click_pos].set_resource_quantity(Flower::MAX);
                    map[click_pos] |= Flag::FLOWER;
                } else {
                }
            }
            MouseButton::Right => {
                if game_state.tree_ammo > 0 && map[click_pos].is_empty() {
                    if !cheat.0 {
                        game_writer.send(GameEvent::Tree(ByteOp::Decrement));
                    }
                    commands.spawn().insert(Tree).insert(Position(click_pos));
                    map[click_pos].set_resource_quantity(Tree::MAX);
                    map[click_pos] |= Flag::TREE;
                } else {
                }
            }
            MouseButton::Middle => {
                if game_state.delete_ammo > 0
                    && (map[click_pos] & Flag::KIND_MASK).intersects(Flag::FLOWER | Flag::TREE)
                {
                    if !cheat.0 {
                        game_writer.send(GameEvent::Delete(ByteOp::Decrement));
                    }
                    map[click_pos].set_resource_quantity(0);
                    map[click_pos] = Flag::EMPTY;
                } else {
                }
            }
            _ => {},
        }
    }
    if key_input.just_pressed(KeyCode::Space) {
        game_writer.send(GameEvent::SpeedChange(SpeedOp::Toggle));
    }

    // NOTE: Disabled saving for game jam submission, breaks if multivac has spawned
    //if key_input.just_pressed(KeyCode::M) {
    //    save_system(map);
    //    info!("saved world map");
    //}

    if key_input.just_pressed(KeyCode::C) {
        cheat.0 = !cheat.0;
        info!("cheats toggled on");
    }
}

pub struct Apocalypse(pub Timer);

/// Bring about the apocalypse
pub fn start_apocalypse(
    mut commands: Commands,
    mut apoc: ResMut<Apocalypse>,
    time_step: Res<TimeStep>,
    mut map: ResMut<WorldMap>,
) {
    let position = IVec2::splat((crate::WORLD_SIZE / 2) as i32);
    map[position] |= Flag::MULTIVAC;
    //draw::ping(&mut commands, position);
    if apoc.0.tick(time_step.into_inner().into()).just_finished() {
        commands
            .spawn()
            .insert(Multivac)
            .insert(Position(position.as_vec2()));
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldMap::new())
            .insert_resource(TimeStep::STOP)
            .insert_resource(Apocalypse(Timer::from_seconds(APOCALYPSE_COUNTDOWN, false)))
            .insert_resource(Cheat(false))
            .add_system_set(
                SystemSet::new()
                    .before(Order::WorldUpdate)
                    .with_system(update_timestep)
                    .with_system(handle_event)
                    .with_system(start_apocalypse)
                    .with_system(animate_despawn_flower)
                    .with_system(despawn_tree)
                    .with_system(despawn_volcano),
            )
            .add_startup_system(WorldMap::initialize_map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn quantity_bits() {
        let mut flowers = Flag::FLOWER;
        assert_eq!(0, flowers.get_resource_quantity());

        flowers.set_resource_quantity(100);
        assert_eq!(100, flowers.get_resource_quantity());

        flowers.set_resource_quantity(1000);
        assert_eq!(1000, flowers.get_resource_quantity());

        flowers.set_resource_quantity(Flag::MAX_RESOURCE_COUNT);
        assert_eq!(Flag::MAX_RESOURCE_COUNT, flowers.get_resource_quantity());
    }
}
