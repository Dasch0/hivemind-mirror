use crate::{
    draw::{self, ping, ping_long},
    world::{self, Flag, WorldMap},
    AppState,
};
/// AI implementation for supercomputer civ
use bevy::{math, prelude::*};
use std::collections::{HashMap, VecDeque};

// clockwise, starting north
const DIRS: [IVec2; 4] = [
    math::const_ivec2!([0, 1]),
    math::const_ivec2!([0, -1]),
    math::const_ivec2!([1, 0]),
    math::const_ivec2!([-1, 0]),
];

pub struct Config {
    location: IVec2,
    route_clock: f32, // in seconds
    gather_clock: f32,
    route_delay: f32,
    gather_rate: u32,
}

impl Config {
    pub fn default() -> Self {
        Self {
            location: IVec2::splat((crate::WORLD_SIZE * 3 / 4) as i32),
            route_clock: 0.1,
            gather_clock: 1.0,
            route_delay: 0.3,
            gather_rate: 50,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchPath {
    //NOTE: data stored as IVec2 so we can wait till the last moment to cast, but still compare for
    // validity
    pub pos: IVec2,
    pub prev: IVec2,
}

impl SearchPath {
    pub fn new(current: IVec2, prev: IVec2) -> Self {
        Self { pos: current, prev }
    }
}

pub enum Dir {
    None,
    North,
    South,
    East,
    West,
}

impl Dir {
    pub fn from_ivec2(d: IVec2) -> Self {
        if d == DIRS[0] {
            Dir::North
        } else if d == DIRS[1] {
            Dir::South
        } else if d == DIRS[2] {
            Dir::East
        } else if d == DIRS[3] {
            Dir::West
        } else {
            Dir::None
        }
    }
}

#[derive(Component)]
pub enum WireKind {
    Disconnect(Dir),
    Connect(Dir, Dir),
}

impl WireKind {
    pub fn from_route(prev: IVec2, pos: IVec2, next: IVec2) -> Self {
        Self::Connect(Dir::from_ivec2(pos - prev), Dir::from_ivec2(pos - next))
    }
}

/// State of the routing process
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultivacState {
    Init,             // startup state only
    InitRoute(IVec2), // a connection at this location
    Route(IVec2),     // a connection at this location
    InFlight,         // connection is being completed this tick
    Search(Option<IVec2>),
    Stop,
    Error,
}

/// the real-time period between multivac state updates
#[derive(Component)]
pub struct Clock(pub Timer);

#[derive(Component, Debug)]
pub struct Multivac {
    /// Origin of the multivac network
    pub origin: IVec2,
    visited: HashMap<IVec2, IVec2>, // visited tile key, path back to origin value
    pub queue: VecDeque<SearchPath>,
}

impl Multivac {
    fn new(origin: IVec2, map: &mut WorldMap) -> Self {
        let multivac = Self {
            origin,
            visited: HashMap::with_capacity(map.w() * map.h()),
            // NOTE: maximum surface area of the search is simultaenously reaching all 4 world edges
            queue: VecDeque::with_capacity(map.w() * 4),
        };

        multivac
    }

    pub fn init(&mut self, map: &WorldMap) -> MultivacState {
        self.visited.clear();
        self.queue.clear();
        // All of origin's neighbors go into the queue
        self.visited.insert(self.origin, IVec2::ZERO);
        for dir in DIRS {
            let neighbor = self.origin + dir;
            match map.get_ivec2(neighbor) {
                Some(_) => {
                    self.queue.push_back(SearchPath::new(neighbor, self.origin));
                    // search twice to get neighbor of neighbors
                    for dir in DIRS {
                        let neighbor_2 = self.origin + dir;
                        self.queue
                            .push_back(SearchPath::new(neighbor_2, self.origin))
                    }
                }
                None => continue,
            }
        }
        MultivacState::Search(None)
    }

    /// search routine for a single clock cycle
    // NOTE: This flavor of BFS is ordered as such:
    //  1. Attempt to 'visit' cell
    //  2. Early exit if it is our destination OR if it is
    //  3. Actually add to visited list and then enqueue neighbors
    pub fn search(&mut self, map: &WorldMap) -> MultivacState {
        let candidate = match self.queue.pop_front() {
            Some(v) => v,
            None => return MultivacState::Stop, // early exit path if queue is empty
        };

        // BFS
        if self.visited.contains_key(&candidate.pos) {
            return MultivacState::Search(None);
        }
        self.visited.insert(candidate.pos, candidate.prev);

        let flags = match map.get_ivec2(candidate.pos) {
            Some(f) => f,
            None => return MultivacState::Search(None),
        };

        // currently on a non-empty tile, can't visit
        if flags.intersects(!Flag::MULTIVAC_FOOD) {
            return MultivacState::Search(None);
        }

        if flags.intersects(Flag::MULTIVAC_FOOD) {
            // things multivac can connect to
            return MultivacState::InitRoute(candidate.pos);
        }
        for dir in DIRS {
            let neighbor = candidate.pos + dir;
            match map.get_ivec2(neighbor) {
                Some(_) => {
                    if !self.visited.contains_key(&neighbor) {
                        self.queue
                            .push_back(SearchPath::new(neighbor, candidate.pos))
                    }
                    {}
                }
                None => continue,
            }
        }
        MultivacState::Search(Some(candidate.pos))
    }

    /// search with the opposite iteration order over the neighbors
    pub fn search_reverse(&mut self, map: &WorldMap) -> MultivacState {
        let candidate = match self.queue.pop_front() {
            Some(v) => v,
            None => return MultivacState::Stop, // early exit path if queue is empty
        };

        // BFS
        if self.visited.contains_key(&candidate.pos) {
            return MultivacState::Search(None);
        }
        self.visited.insert(candidate.pos, candidate.prev);

        let flags = match map.get_ivec2(candidate.pos) {
            Some(f) => f,
            None => return MultivacState::Search(None),
        };

        // currently on a non-empty tile, can't visit
        if flags.intersects(!Flag::MULTIVAC_FOOD & Flag::KIND_MASK) {
            return MultivacState::Search(None);
        }

        if flags.intersects(Flag::MULTIVAC_FOOD) {
            // things multivac can connect to
            return MultivacState::InitRoute(candidate.pos);
        }

        for dir in DIRS.iter().rev() {
            let neighbor = candidate.pos + *dir;
            match map.get_ivec2(neighbor) {
                Some(_) => {
                    if !self.visited.contains_key(&neighbor) {
                        self.queue
                            .push_back(SearchPath::new(neighbor, candidate.pos))
                    }
                    {}
                }
                None => continue,
            }
        }
        MultivacState::Search(Some(candidate.pos))
    }
}

pub fn setup(
    mut commands: Commands,
    config: Res<Config>,
    map: ResMut<WorldMap>,
    query: Query<(Entity, &world::Multivac, &world::Position), Without<Multivac>>,
) {
    let map = map.into_inner();
    for (entity, _, position) in query.iter() {
        commands
            .entity(entity)
            .insert(Multivac::new(position.0.as_ivec2(), map))
            .insert(Clock(Timer::from_seconds(config.route_clock, false)))
            .insert(MultivacState::Init);
        map[position.0] |= Flag::MULTIVAC;

        for dir in DIRS.iter().rev() {
            let neighbor = config.location + *dir;
            match map.get_ivec2(neighbor) {
                Some(_) => {
                    commands
                        .spawn()
                        .insert(world::Wire)
                        .insert(WireKind::Disconnect(Dir::from_ivec2(*dir)))
                        .insert(Clock(Timer::from_seconds(config.route_clock, false)))
                        .insert(world::Position(neighbor.as_vec2()))
                        .insert(MultivacState::Init);
                }
                None => continue,
            }
        }
    }
}

pub fn update(
    mut commands: Commands,
    map_res: ResMut<WorldMap>,
    time_step: Res<world::TimeStep>,
    config: Res<Config>,
    mut multivac_query: Query<(&mut Multivac, &mut Clock, &mut MultivacState)>,
) {
    let map = map_res.into_inner();

    let time_step = time_step.into_inner();
    for (mut multivac, mut clock, state) in multivac_query.iter_mut() {
        if clock.0.tick(time_step.into()).just_finished() {
            let state = state.into_inner();
            *state = match state {
                MultivacState::Init => multivac.init(map),
                MultivacState::InitRoute(p) => {
                    map[*p] |= Flag::OUTPOST;
                    ping(&mut commands, *p);
                    commands
                        .spawn()
                        .insert(world::Outpost)
                        .insert(Clock(Timer::from_seconds(config.gather_clock, true)))
                        .insert(world::Position(p.as_vec2()));
                    MultivacState::Route(*p)
                }
                // route all in one tick to prevent pathing errors from player clicks
                MultivacState::Route(p) => {
                    // backtrack by following visited, place down wires. If prev somehow isn't in visited, early exit
                    let mut state = MultivacState::InFlight;
                    let mut prev = *p;
                    // punch out if we are taking too long
                    let mut distance_count = 0;
                    while state == MultivacState::InFlight {
                        distance_count += 1;
                        let pos = match multivac.visited.get(&prev) {
                            Some(p) => *p,
                            None => {
                                state = MultivacState::Error;
                                break;
                            }
                        };
                        let next = match multivac.visited.get(&pos) {
                            Some(p) => *p,
                            None => {
                                state = MultivacState::Error;
                                break;
                            }
                        };

                        let pos_flags = match map.get_ivec2(pos) {
                            Some(f) => f,
                            None => {
                                state = MultivacState::Error;
                                break;
                            }
                        };
                        let next_flags = match map.get_ivec2(pos) {
                            Some(f) => f,
                            None => {
                                state = MultivacState::Error;
                                break;
                            }
                        };

                        if next_flags.intersects(Flag::MULTIVAC) {
                            state = MultivacState::Stop;
                        }
                        if !pos_flags.is_empty() {
                            // map changed since searched
                            state = MultivacState::Error;
                            break;
                        };
                        if distance_count >= crate::WORLD_SIZE * crate::WORLD_SIZE {
                            state = MultivacState::Error;
                            break;
                        }

                        map[pos] |= Flag::WIRE; // @NOTE: pos already checked above to get flags
                        commands
                            .spawn()
                            .insert(world::Wire)
                            .insert(world::Wire)
                            .insert(WireKind::from_route(prev, pos, next))
                            .insert(world::Position(pos.as_vec2()))
                            .insert(draw::Delay(Timer::from_seconds(
                                distance_count as f32 * config.route_delay,
                                false,
                            )));
                        ping_long(&mut commands, pos);
                        prev = pos;
                    }
                    state
                }
                MultivacState::InFlight => MultivacState::Error, // should not be caught in flight between clock cycles
                MultivacState::Search(_) => {
                    let mut state = MultivacState::Stop;
                    for _ in 0..multivac.queue.len() / 2 {
                        state = multivac.search(map);
                        match state {
                            MultivacState::Search(Some(p)) => ping(&mut commands, p),
                            MultivacState::Search(None) => {}
                            _ => break,
                        };

                        state = multivac.search_reverse(map);
                        match state {
                            MultivacState::Search(Some(p)) => ping(&mut commands, p),
                            MultivacState::Search(None) => {}
                            _ => break,
                        };
                    }
                    state
                }
                MultivacState::Stop => multivac.init(map), // TODO: placeholder
                MultivacState::Error => multivac.init(map), //TODO: placeholder
            };

            // TODO: put this in config
            let next_clock = match state {
                MultivacState::Init => 50.0,
                MultivacState::InitRoute(_) => 0.0,
                MultivacState::Route(_) => 0.5,
                MultivacState::InFlight => 0.1,
                MultivacState::Search(_) => 0.1,
                MultivacState::Stop => 50.0,
                MultivacState::Error => 50.0,
            };
            clock.0 = Timer::from_seconds(next_clock, false);
        }
    }
}

pub fn promote_outpost(
    mut commands: Commands,
    time_step: Res<world::TimeStep>,
    config: Res<Config>,
    mut map: ResMut<WorldMap>,
    mut query: Query<(Entity, &mut Clock, &world::Position), With<world::Outpost>>,
) {
    let time_step = time_step.into_inner();
    for (entity, mut clock, pos) in query.iter_mut() {
        if clock.0.tick(time_step.into()).just_finished() {
            // sanity check that pos is not out of range
            if let None = map.get_vec2(pos.0) {
                continue;
            }

            // if food depleted, become a multivac spawner
            if !map[pos.0].intersects(Flag::MULTIVAC_FOOD) {
                commands.entity(entity).despawn();
                map[pos.0] = Flag::MULTIVAC; // the hard assignment is meant to remove the outpost flag
                commands
                    .spawn()
                    .insert(world::Multivac)
                    .insert(world::Position(pos.0));
                draw::ping_long(&mut commands, pos.0.as_ivec2());
            // eat the food
            } else {
                let resource = map[pos.0].get_resource_quantity();

                if resource == 0 {
                    map[pos.0] &= !Flag::MULTIVAC_FOOD;
                } else {
                    map[pos.0].set_resource_quantity(
                        resource - std::cmp::min(resource, config.gather_rate),
                    );
                }
            }
        }
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Config::default()).add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(setup)
                .with_system(update)
                .with_system(promote_outpost),
        );
    }
}
