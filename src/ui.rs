use crate::{
    grid::{self, GRID_COUNT},
    hud::HudContext,
    RotationEvent,
};
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::primitives::Frustum,
    window::{WindowFocused, WindowResized},
};

use crate::camera::CoordinateVectors;
use bevy_ecs_tilemap::prelude::*;

pub struct WorldClickEvent {
    pub pos: Vec2,
    pub btn: MouseButton,
}

#[derive(Default)]
pub struct UiContext {
    pub scroll_locked: bool,
    pub should_scroll: bool,
    pub window_size: Vec2,
    pub cursor_position: Vec2,
    pub coord_vecs: CoordinateVectors,
    pub hovered_tile: Option<Entity>,
    pub prev_hovered_pos: Option<TilePos>,
    // matrix for undoing the projection and camera transform
    pub ndc_to_world: Mat4,
    pub cursor_edge: Option<CursorEdge>,
}

const EDGE_EPSILON: f32 = 10.0;
#[derive(Debug, Clone, Copy)]
pub enum CursorEdge {
    N,
    E,
    S,
    W,
    NE,
    SE,
    SW,
    NW,
}

impl CursorEdge {
    #[inline]
    pub fn north(self) -> bool {
        matches!(self, CursorEdge::N | CursorEdge::NE | CursorEdge::NW)
    }
    #[inline]
    pub fn south(self) -> bool {
        matches!(self, CursorEdge::S | CursorEdge::SE | CursorEdge::SW)
    }
    #[inline]
    pub fn east(self) -> bool {
        matches!(self, CursorEdge::E | CursorEdge::NE | CursorEdge::SE)
    }
    #[inline]
    pub fn west(self) -> bool {
        matches!(self, CursorEdge::W | CursorEdge::NW | CursorEdge::SW)
    }
}

pub fn cursor_grab_system(mut windows: ResMut<Windows>, fcs: Option<Res<WindowFocused>>) {
    if let Some(event) = fcs {
        if event.focused {
            let _window = windows.get_primary_mut().unwrap();
            //window.set_cursor_lock_mode(true);
        }
    }
}

pub fn cache_window_attrs_setup(
    mut ui_context: ResMut<UiContext>,
    windows: Res<Windows>,
    query: Query<(&Camera, &GlobalTransform), With<Frustum>>,
) {
    let window = windows.get_primary().unwrap();
    ui_context.window_size.x = window.width();
    ui_context.window_size.y = window.height();
    println!("{:?}", ui_context.window_size);
    let (camera, global_transform) = query.single();
    ui_context.ndc_to_world =
        global_transform.compute_matrix() * camera.projection_matrix.inverse();
}

pub fn cache_window_attrs_system(
    mut ui_ctx: ResMut<UiContext>,
    mut windows: ResMut<Windows>,
    mut rsz: EventReader<WindowResized>,
    // gate for edge pan detection and mouse position
    mut movement: EventReader<MouseMotion>,
    mut rts: EventReader<RotationEvent>,
    query: Query<(&Camera, &GlobalTransform), With<Frustum>>,
) {
    let mut force_middle = None;
    if let Some(event) = rsz.iter().next() {
        ui_ctx.window_size.x = event.width;
        ui_ctx.window_size.y = event.height;
        println!(
            "[cache_window_attrs_system] Resized: {:?}",
            ui_ctx.window_size
        );
        let window = windows.get_primary_mut().unwrap();
        // window.set_cursor_position(ui_ctx.window_size / 2.0);
        // force cursor to middle of screen on resize to avoid annoying things
        force_middle = Some(ui_ctx.window_size / 2.0);
    }

    if rts.iter().next().is_some() {
        if let Some((camera, global_transform)) = query.iter().next() {
            ui_ctx.ndc_to_world =
                global_transform.compute_matrix() * camera.projection_matrix.inverse();
        }
    }

    if movement.iter().next().is_some() || force_middle.is_some() {
        let window = windows.get_primary().unwrap();

        ui_ctx.cursor_position = force_middle
            .unwrap_or_else(|| window.cursor_position().unwrap_or_else(|| Vec2::splat(0.)));

        let c_pos = ui_ctx.cursor_position;
        let delta_pos = ui_ctx.window_size - c_pos;

        ui_ctx.cursor_edge = None;
        if c_pos.x < EDGE_EPSILON {
            ui_ctx.cursor_edge = Some(CursorEdge::W);
        } else if delta_pos.x < EDGE_EPSILON {
            ui_ctx.cursor_edge = Some(CursorEdge::E);
        }

        if c_pos.y < EDGE_EPSILON {
            match ui_ctx.cursor_edge {
                Some(CursorEdge::W) => ui_ctx.cursor_edge = Some(CursorEdge::SW),
                Some(CursorEdge::E) => ui_ctx.cursor_edge = Some(CursorEdge::SE),
                None => ui_ctx.cursor_edge = Some(CursorEdge::S),
                _ => unreachable!(),
            }
        } else if delta_pos.y < EDGE_EPSILON {
            match ui_ctx.cursor_edge {
                Some(CursorEdge::W) => ui_ctx.cursor_edge = Some(CursorEdge::NW),
                Some(CursorEdge::E) => ui_ctx.cursor_edge = Some(CursorEdge::NE),
                None => ui_ctx.cursor_edge = Some(CursorEdge::N),
                _ => unreachable!(),
            }
        }
        ui_ctx.cursor_edge = None;
    }
}

pub fn cursor_system(
    mut ui_ctx: ResMut<UiContext>,
    hud_ctx: Res<HudContext>,
    mut tile_query: Query<&mut Tile>,
    mut map_query: MapQuery,

    mouse_button_input: Res<Input<MouseButton>>,
    mut click_dispatcher: EventWriter<WorldClickEvent>,
) {
    if hud_ctx.hud_hovered {
        // @CLEANUP copy paste from later in function
        if let Some(hovered_tile) = ui_ctx.hovered_tile {
            let mut hovered = tile_query.get_mut(hovered_tile).unwrap();
            hovered.visible = false;
            if let Some(prev_pos) = ui_ctx.prev_hovered_pos {
                map_query.notify_chunk_for_tile(prev_pos, 0u16, u16::MAX);
            }
        }
        // bail
        return;
    }

    // check if the cursor is inside the window and get its position
    // get the size of the window
    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (ui_ctx.cursor_position / ui_ctx.window_size) * 2.0 - Vec2::ONE;

    // convert ndc through iso, world, and finally get the tile
    let iso_pos = ui_ctx.ndc_to_world.project_point3(ndc.extend(-1.0));
    let world_pos = grid::iso_to_world(iso_pos);
    let tile_pos = grid::tile_pos(world_pos);

    //eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);
    //eprintln!("grid coords: {:?}", world_space_to_tile_pos(world_pos));
    let mut notify_chunk = false;
    let tile = match map_query.get_tile_entity(tile_pos, 0u16, u16::MAX) {
        Ok(tile_entity) => {
            let mut tile = tile_query.get_mut(tile_entity).unwrap();
            tile.visible = true;
            notify_chunk = true;
            Some(tile_entity)
        }
        Err(_) => None,
    };
    if let Some(hovered_tile) = ui_ctx.hovered_tile {
        if let Some(inner_tile) = tile {
            if inner_tile != hovered_tile {
                //println!("[cursor_system] new hovered_tile: {:?}", tile_pos);
                let mut hovered = tile_query.get_mut(hovered_tile).unwrap();
                hovered.visible = false;
            } else {
                notify_chunk = false;
            }
        } else {
            let mut hovered = tile_query.get_mut(hovered_tile).unwrap();
            hovered.visible = false;
            notify_chunk = true;
        }
    }
    if notify_chunk {
        map_query.notify_chunk_for_tile(tile_pos, 0u16, u16::MAX);
        if let Some(prev_pos) = ui_ctx.prev_hovered_pos {
            map_query.notify_chunk_for_tile(prev_pos, 0u16, u16::MAX);
        }
    }
    if tile.is_some() {
        ui_ctx.prev_hovered_pos = Some(tile_pos);

        if mouse_button_input.just_pressed(MouseButton::Left) {
            click_dispatcher.send(WorldClickEvent {
                pos: world_pos,
                btn: MouseButton::Left,
            });
        }

        if mouse_button_input.just_pressed(MouseButton::Right) {
            click_dispatcher.send(WorldClickEvent {
                pos: world_pos,
                btn: MouseButton::Right,
            });
        }

        if mouse_button_input.just_pressed(MouseButton::Middle) {
            click_dispatcher.send(WorldClickEvent {
                pos: world_pos,
                btn: MouseButton::Middle,
            });
        }
    }

    ui_ctx.hovered_tile = tile;
}
