mod camera;
mod draw;
mod game;
mod grid;
mod hivemind;
mod hud;
mod multivac;
mod story;
mod texture;
mod ui;
mod util;
mod world;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub mod prelude {
    pub use crate::game::{ByteOp, GameEvent, GameState, SpeedOp};
    pub use crate::hud::HudUpdateEvent;
    pub use crate::ui::WorldClickEvent;
    pub use crate::world::TimeStep;
}

/// Top level convenience defs, used by multiple modules
pub const WORLD_SIZE: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
pub enum GameStage {
    PreUpdate,
    Update,
    PreRender,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Load,
    Playing,
}

pub struct RotationEvent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum SystemLabel {
    UiCache,
    CameraMove,
    HudGatekeep,
    LoadTextures,
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    {
        let _window = windows.get_primary_mut().unwrap();
    }
}

// NOTE: This cannot be the correct way to make egui windows transparent, but I cannot find
// documentation on a different method
//fn setup_console_style(mut egui_ctx: ResMut<EguiContext>) {
//    let mut fonts = FontDefinitions::default();
//    fonts.font_data.insert(
//        "my_font".to_owned(),
//        FontData::from_static(include_bytes!("../assets/fonts/monogram.ttf")),
//    );
//
//    // Put my font first (highest priority):
//    fonts
//        .fonts_for_family
//        .get_mut(&FontFamily::Monospace)
//        .unwrap()
//        .insert(0, "my_font".to_owned());
//
//    fonts
//        .fonts_for_family
//        .get_mut(&FontFamily::Proportional)
//        .unwrap()
//        .insert(0, "my_font".to_owned());
//
//    egui_ctx.ctx_mut().set_fonts(fonts);
//
//    let style = egui_ctx.ctx_mut().style();
//    let mut visuals = style.visuals.clone();
//    visuals.selection.bg_fill[3] = 200;
//    visuals.faint_bg_color[3] = 200;
//    visuals.extreme_bg_color[3] = 200;
//    visuals.code_bg_color[3] = 200;
//    visuals.widgets.noninteractive.bg_fill[3] = 200;
//    visuals.widgets.inactive.bg_fill[3] = 200;
//    visuals.widgets.hovered.bg_fill[3] = 200;
//    visuals.widgets.active.bg_fill[3] = 200;
//    visuals.widgets.open.bg_fill[3] = 200;
//
//    let new_style = egui::style::Style {
//        visuals,
//        ..Default::default()
//    };
//
//    egui_ctx.ctx_mut().set_style(new_style);
//}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1920.0,
            height: 1080.0,
            title: String::from("Hivemind"),
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BEIGE))
        .insert_resource(game::Time(0.0))
        .insert_resource(game::GameTimer(Timer::from_seconds(1.0, true)))
        .init_resource::<texture::TextureHandles>()
        .init_resource::<texture::TextureAtlases>()
        .init_resource::<hud::HudContext>()
        .init_resource::<game::GameState>()
        .init_resource::<game::ReloadTimer>()
        //.insert_resource(ConsoleConfiguration {
        //    top_pos: 10.0,
        //    left_pos: 10.0,
        //    height: 400.0,
        //    width: 400.0,
        //    ..Default::default()
        //})
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(world::Plugin)
        .add_plugin(draw::Plugin { debug: false })
        .add_plugin(hivemind::Plugin)
        .add_plugin(multivac::Plugin)
        //.add_startup_system_to_stage(StartupStage::Startup, setup_console_style)
        //.add_plugin(ConsolePlugin) // FIXME: could not work around auto expanding console window,
        //disabling for now
        //.add_plugin(story::Plugin) // FIXME: could not work around auto expanding console window,
        //disabling for now
        .add_event::<RotationEvent>()
        .add_event::<ui::WorldClickEvent>()
        .add_event::<hud::HudUpdateEvent>()
        .add_event::<game::GameEvent>()
        .insert_resource(ui::UiContext {
            coord_vecs: camera::CoordinateVectors::new(),
            ..Default::default()
        })
        .add_state(AppState::Load)
        .add_startup_system_to_stage(StartupStage::Startup, setup)
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            ui::cache_window_attrs_system.chain(
                hud::gatekeep_cursor_system
                    .chain(camera::movement)
                    .chain(ui::cursor_system),
            ),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Load)
                .with_system(texture::load_textures)
                .label(SystemLabel::LoadTextures),
        )
        .add_system_set(SystemSet::on_update(AppState::Load).with_system(texture::check_textures))
        .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(grid::setup))
        .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(hud::setup))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(hud::hud_update_system))
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(
                    ui::cache_window_attrs_system.chain(
                        hud::gatekeep_cursor_system
                            .chain(camera::movement)
                            .chain(ui::cursor_system),
                    ),
                )
                .with_system(hud::button_system),
        )
        .add_system(texture::set_texture_filters_to_nearest)
        .add_system(game::handle_game_events)
        .add_system(game::reload)
        .add_system(game::game_time)
        .run();
}
