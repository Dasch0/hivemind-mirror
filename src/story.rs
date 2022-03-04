/// Implementation of story and story event triggers. As well as all source text
use crate::AppState;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_console::PrintConsoleLine;

use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin};

use crate::game;

/// trigger the next story element
pub struct Trigger(pub usize);

pub struct Text;

const INFO: &'static str = "To control the ship: 
 - [WASD] or [MOUSE4] to move the camera
 - [Z], [X], and [SCROLL WHEEL] control the camera zoom
 - [MOUSE1] to place flowers on the map
 - [MOUSE2] to place trees on the map
 - [MOUSE3] to remove a flower or tree from the map
 - [T] will enable the observatory scanner, showing any interesting signals present in the world
 - [SPACE] (or the play button in the top right corner), controls the simulation speed
";

static TEXT_LINES: &[(&'static str, &'static str, f32)] = &[
("Tortoise", "Welcome to GEB observatory, glad you finally made it...\n", 5.0),
("Crab", "We've been very busy monitoring a newly discovered alien species. And we we urgently need your help.", 5.0),
("Tortoise", "These aliens, they appear to function as a sort of hivemind, split into three different colonies, but we don't know-", 0.0),
("Crab:", "Problem is, they're nearly extinct, each down to their last drone in fact!", 2.0),
("Crab:", "While we conduct our research, we need you to help keep these aliens alive...", 2.0),
("Tortoise:", "The observatory can spawn resources, create barriers, or clear pathways on their world to help keep the colonies alive", 1.0),
("Tortoise:", "Remember, we need all three colonies healthy in order to continue our work. If you need help, type 'info' into the Observer Console to list the controls", 1.0),
("Crab", "And don't forget Tortoise, [Q] and [E] rotate the observatory!", 7.0),
("Tortoise:", "Yes...but why would literally anyone ever want to do that?", 1.0),
("Crab:", "I really have no idea, but it's in the onboarding guide...", 25.0),
("Tortoise:", "wait...what is THAT", 0.5),
("Crab:", "what?", 1.25),
("Tortoise:", "There - in the middle of the map..", 0.6),
("Crab:", "Some sort of robot, maybe?", 7.0),
("Tortoise:", "Oh no...this is bad...just try to keep the colonies safe for now", -1.0),
];

static WARNING: &'static str =
    "Crab: Look out! A colony is dangerously close to running out of food";

pub fn warning(console_line: &mut EventWriter<PrintConsoleLine>, time: &Res<game::Time>) {
    console_line.send(PrintConsoleLine::new(WARNING.to_string()));
}

pub struct GameOver(bool, Timer);

pub struct GameOverEvent;

/// top level system to display story text in the console
///
/// Format is "speaker: text"
pub fn game_over(
    mut timer: ResMut<TextTimer>,
    mut game_over: ResMut<GameOver>,
    console_line: &mut EventWriter<PrintConsoleLine>,
    time: &Res<game::Time>,
    mut event: EventReader<GameOverEvent>,
) {
    for _e in event.iter() {
        if !game_over.0 {
            // first game over event
            // purge main timer tick on the first event recv
            timer.0.set_duration(Duration::from_secs_f32(0.0));
            timer.0.set_repeating(false);
            timer.0.tick(Duration::from_secs_f32(0.1));

            // print game over message
            console_line.send(PrintConsoleLine::new(format!("Tortoise: A hivemind colony has gone extinct. Thanks to your efforts, we had a whole {} extra minutes to study these creatures", time.0 as u128 / 60)));
            console_line.send(PrintConsoleLine::new(format!(
                "Crab: your final score was {:02}:{:02}:{:02}",
                time.0 as u128 / 3600,
                (time.0 as u128 / 60) % 60,
                time.0 as u128 % 60
            )));
            console_line.send(PrintConsoleLine::new(format!("Crab: by the way, on your next play-through you can type 'enable cheats' into the console, this will disable resource limits and colony death")));
            console_line.send(PrintConsoleLine::new(format!("Tortoise: You probably should have mentioned that before our research subjects went extinct...")));
        } else {
        } // ignore subsequent game over events
    }
}

pub struct TextTimer(pub Timer);

pub fn timer(
    mut timer: ResMut<TextTimer>,
    time_step: Res<TimeStep>,
    mut console_line: EventWriter<PrintConsoleLine>,
) {
    let time_step = time_step.into_inner();
    if timer
        .0
        .tick(Duration::from_secs_f32(time_step.0)) //
        .just_finished()
    {
        let index = timer.0.times_finished();

        if (index as usize) < TEXT_LINES.len() {
            console_line.send(PrintConsoleLine::new(format!(
                "{} {}",
                TEXT_LINES[index as usize].0, TEXT_LINES[index as usize].1
            )));
            timer
                .0
                .set_duration(Duration::from_secs_f32(TEXT_LINES[index as usize].2));
        }
    }
}

/// get controls
#[derive(ConsoleCommand)]
#[console_command(name = "info")]
pub struct InfoCommand;

pub fn info_system(mut info: ConsoleCommand<InfoCommand>) {
    if let Some(InfoCommand) = info.take() {
        reply!(info, "{}", INFO);
    }
}

/// enable cheats
#[derive(ConsoleCommand)]
#[console_command(name = "enable cheats")]
pub struct CheatCommand;

pub fn cheat_system(mut cheat: ConsoleCommand<CheatCommand>) {
    if let Some(CheatCommand) = cheat.take() {
        reply!(cheat, "cheats enabled.");
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Text)
            .insert_resource(TextTimer(Timer::from_seconds(7.0, true)))
            .add_event::<Trigger>()
            .add_event::<GameOverEvent>()
            .add_system_set(SystemSet::on_update(AppState::Playing).with_system(timer))
            .add_console_command::<InfoCommand, _, _>(info_system)
            .add_console_command::<CheatCommand, _, _>(cheat_system);
    }
}
