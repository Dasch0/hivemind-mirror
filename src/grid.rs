/// Information about the isometric grid the game is drawn on
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{texture::TextureHandles, util, WORLD_SIZE};

pub const GRID_COUNT: u32 = WORLD_SIZE as u32;
pub const TILE_WIDTH: f32 = 102.;
pub const TILE_HEIGHT: f32 = 51.;
pub const TILE_SIZE: (f32, f32) = (TILE_WIDTH, TILE_HEIGHT);
pub const SPRITE_SHEET_TILE_COUNT: f32 = 10.;
pub const CHUNKS: (u32, u32) = (2, 2);
pub const CHUNK_SIZE: (u32, u32) = (GRID_COUNT / CHUNKS.0, GRID_COUNT / CHUNKS.1);
pub const ORIGIN_OFFSET: f32 = TILE_SIZE.1 / 2.0 * GRID_COUNT as f32;

// @NOTE: Going forward, I'm enforcing that all world sprites (iso perspective) are a standard size at export time.
// This prevents us from having to track the sprite offsets and calculate on the fly. This also
// makes sure we have equally sized pixels for all sprites drawn in the world
//
// World sprites are:
//  * Aligned to 3d isometric tiles that are 26 x 26 x 26 'hexels'
//  * each hexel is 2 x 2 pixels
//  * The resulting size of the final image is exactly 102 x 104 pixels
//
// The only exception to this is the sprite sheet used in `crate::grid` as they are drawn
// separately by the tile system with their own fixed dimensions. Generally though the tile sprites
// should just be half the height of world sprites
//
// If we do still end up needing dyanmic offset we can bring the GridOffset back
/// Helper function to project world position to isometric
pub fn world_to_iso(pos: Vec2) -> Vec3 {
    // FIXME: naive attempt at fixing z fighting
    let ordered_z = pos.y + pos.x;
    let (x, y) = (pos.x, pos.y);
    let px = (x - y) * (TILE_WIDTH / 2.0);
    let py = (x + y) * (TILE_HEIGHT / 2.0);

    Vec3::new(px, -py + ORIGIN_OFFSET, ordered_z)
}

pub fn world_to_iso_no_offset(pos: Vec2) -> Vec3 {
    // FIXME: naive attempt at fixing z fighting
    let ordered_z = pos.y + pos.x;
    let (x, y) = (pos.x - 0.5, pos.y - 0.5);
    let px = (x - y) * (TILE_WIDTH / 2.0);
    let py = (x + y) * (TILE_HEIGHT / 2.0);

    Vec3::new(px, -py + ORIGIN_OFFSET, ordered_z)
}

/// Helper function converting an iso position to world position
/// GENERICALLY
/// x = (y_world / (grid_len/2)) + (x_world / (grid_len * y_to_x_pixels_ratio))
/// y = (y_world / (grid_len/2)) - (x_world / (grid_len))
/// i think?
/// assuming x_world and y_world have been translated such that 0,0 is the same as grid coords and
/// the direction of x and y share signs
// @TODO this assumes that the ratio of the pixels of x vs y as well as the ratio of grid cells vs
// pixels per grid cell
pub fn iso_to_world(iso: Vec3) -> Vec2 {
    let x = iso.x;
    let y = -(iso.y - ORIGIN_OFFSET);
    let pos = Vec2::new(
        (y + (x / 2.0)) * 2.0 / TILE_WIDTH,
        (y - (x / 2.0)) / TILE_HEIGHT,
    );

    /*
    println!("[y] {}", y);
    println!("[undo] {}", pos);
    pos.x = (y / GRID_COUNT_OVER_2_F32) + (x / GRID_COUNT_F32);
    pos.y = (y / GRID_COUNT_OVER_2_F32) - (x / GRID_COUNT_F32);

    println!("[shadowboxing] {}", grid);
    */

    pos
}

/// Helper function to get the bevy_ecs_tilemap::TilePos from world position
pub fn tile_pos(pos: Vec2) -> TilePos {
    if pos.x < 0. || pos.y < 0. {
        util::unlikely(true);
        TilePos(u32::MAX, u32::MAX)
    } else {
        TilePos(pos.x as u32, pos.y as u32)
    }
}

pub fn setup(
    mut commands: Commands,
    mut map_query: MapQuery,
    texture_handles: Res<TextureHandles>,
) {
    let texture_handle = texture_handles["tiles"].clone();

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut map_settings = LayerSettings::new(
        MapSize(CHUNKS.0, CHUNKS.1),
        ChunkSize(CHUNK_SIZE.0, CHUNK_SIZE.1),
        TileSize(TILE_SIZE.0, TILE_SIZE.1),
        TextureSize(TILE_SIZE.0 * SPRITE_SHEET_TILE_COUNT, TILE_SIZE.1),
    );
    map_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    // Layer 0
    let (mut layer_0, layer_0_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings, 0u16, 0u16);
    map.add_layer(&mut commands, 0u16, layer_0_entity);

    let mut rng = SmallRng::from_entropy();
    for x in 0..GRID_COUNT {
        for y in 0..GRID_COUNT {
            let _ = layer_0.set_tile(
                TilePos(x, y),
                Tile {
                    texture_index: rng.gen_range(0..9 as u16),
                    ..Default::default()
                }
                .into(),
            );
        }
    }

    map_query.build_layer(&mut commands, layer_0, texture_handle.clone());

    let (mut layer_cursor, layer_cursor_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings, 0u16, 0u16);
    map.add_layer(&mut commands, u16::MAX, layer_cursor_entity);

    layer_cursor.fill(
        TilePos(0, 0),
        TilePos(GRID_COUNT, GRID_COUNT),
        Tile {
            texture_index: 9,
            visible: false,
            ..Default::default()
        }
        .into(),
    );
    map_query.build_layer(&mut commands, layer_cursor, texture_handle);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, ORIGIN_OFFSET.ceil(), -1.0))
        .insert(GlobalTransform::default());
}
