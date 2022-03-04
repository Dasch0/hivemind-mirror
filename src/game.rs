use crate::prelude::*;
use bevy::prelude::*;

pub struct GameState {
    pub flower_ammo: u8,
    pub tree_ammo: u8,
    pub delete_ammo: u8,
}

impl GameState {
    pub const MAX_AMMO: u8 = 3;
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            flower_ammo: Self::MAX_AMMO,
            tree_ammo: Self::MAX_AMMO,
            delete_ammo: Self::MAX_AMMO,
        }
    }
}

/// game time, how long since start in seconds
pub struct Time(pub f32);
pub struct GameTimer(pub Timer);

#[derive(Component)]
pub struct UsesTime;

#[derive(Clone, Copy)]
pub enum ByteOp {
    Increment,
    Decrement,
    Set(u8),
}

#[derive(Clone, Copy)]
pub enum SpeedOp {
    Toggle,
    Set(TimeStep),
}

pub enum GameEvent {
    Flower(ByteOp),
    Tree(ByteOp),
    Delete(ByteOp),
    SpeedChange(SpeedOp),
}

impl ByteOp {
    pub fn operate(&self, operand: &mut u8) -> u8 {
        match self {
            ByteOp::Increment => *operand += 1,
            ByteOp::Decrement => *operand -= 1,
            ByteOp::Set(val) => *operand = *val,
        }
        *operand
    }
}

pub struct ReloadTimer(pub Timer);

impl Default for ReloadTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(10.0, true))
    }
}

pub fn reload(
    mut reload_timer: ResMut<ReloadTimer>,
    game_state: Res<GameState>,
    time_step: Res<TimeStep>,
    mut game_events: EventWriter<GameEvent>,
) {
    if reload_timer
        .0
        .tick(time_step.into_inner().into())
        .just_finished()
    {
        info!("reloading");
        if game_state.flower_ammo < GameState::MAX_AMMO {
            game_events.send(GameEvent::Flower(ByteOp::Increment));
        }

        if game_state.tree_ammo < GameState::MAX_AMMO {
            game_events.send(GameEvent::Tree(ByteOp::Increment));
        }

        if game_state.delete_ammo < GameState::MAX_AMMO {
            game_events.send(GameEvent::Delete(ByteOp::Increment));
        }
    }
}

pub fn handle_game_events(
    mut hud_events: EventWriter<HudUpdateEvent>,
    mut game_events: EventReader<GameEvent>,
    mut time_step: ResMut<TimeStep>,
    mut game_state: ResMut<GameState>,
) {
    for event in game_events.iter() {
        match *event {
            GameEvent::Flower(op) => {
                hud_events.send(HudUpdateEvent::FlowerMeter(
                    op.operate(&mut game_state.flower_ammo),
                ));
            }
            GameEvent::Tree(op) => {
                hud_events.send(HudUpdateEvent::TreeMeter(
                    op.operate(&mut game_state.tree_ammo),
                ));
            }
            GameEvent::Delete(op) => {
                hud_events.send(HudUpdateEvent::DeleteMeter(
                    op.operate(&mut game_state.delete_ammo),
                ));
            }
            GameEvent::SpeedChange(op) => match op {
                SpeedOp::Toggle => {
                    hud_events.send(HudUpdateEvent::SpeedChange(time_step.toggle()));
                }
                SpeedOp::Set(_) => todo!(),
            },
        }
    }
}

pub fn game_time(
    mut game_time: ResMut<Time>,
    time_step: Res<TimeStep>,
    mut timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<UsesTime>>,
) {
    let time_step = time_step.into_inner();
    for mut text in query.iter_mut() {
        // if this fails, another text element besides game timers was added
        if timer.0.tick(time_step.into()).just_finished() {
            game_time.0 += 1.0;
            text.sections[0].value = format!("{:02}:{:02}:{:02}", game_time.0 as u128 / 3600, (game_time.0 as u128 / 60) % 60, game_time.0 as u128 % 60);
        }
    }
}
