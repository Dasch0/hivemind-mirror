use crate::hud::HudUpdateEvent;
use crate::AppState;
use bevy::asset::LoadState;
use bevy::{prelude::*, render::render_resource::TextureUsages};

use std::collections::HashMap;

pub type TextureHandles = HashMap<String, Handle<Image>>;

pub type TextureAtlases = HashMap<String, Handle<TextureAtlas>>;

pub fn load_textures(mut texture_handles: ResMut<TextureHandles>, asset_server: Res<AssetServer>) {
    let textures_all = asset_server.load_folder("textures").unwrap();
    for texture in textures_all {
        let handle_path = asset_server.get_handle_path(texture.clone()).unwrap();
        let handle_string = handle_path
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .into(); // FIXME: this cannot be correct way to get string from a path?
        texture_handles.insert(handle_string, texture.typed());
    }

    info!("loading textures into hashmap: {:#?}", texture_handles);
}

pub fn check_textures(
    mut state: ResMut<State<AppState>>,
    game_state: Res<crate::game::GameState>,
    texture_handles: ResMut<TextureHandles>,
    asset_server: Res<AssetServer>,
    mut hud_writer: EventWriter<HudUpdateEvent>,
    mut texture_atlas_map: ResMut<TextureAtlases>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(&mut texture_handles.values().map(|handle| handle.id))
    {
        state.set(AppState::Playing).unwrap();
        info!("loaded textures into hashmap: {:#?}", texture_handles);
        
        hud_writer.send(HudUpdateEvent::FlowerMeter(game_state.flower_ammo));
        hud_writer.send(HudUpdateEvent::TreeMeter(game_state.tree_ammo));
        hud_writer.send(HudUpdateEvent::DeleteMeter(game_state.delete_ammo));

        let bee_atlas = TextureAtlas::from_grid(
            texture_handles["bee_y-sheet"].clone(),
            Vec2::new(102., 104.),
            4,
            1,
        );
        texture_atlas_map.insert("bee_y".into(), texture_atlases.add(bee_atlas));

        let bee_atlas = TextureAtlas::from_grid(
            texture_handles["bee_m-sheet"].clone(),
            Vec2::new(102., 104.),
            4,
            1,
        );
        texture_atlas_map.insert("bee_m".into(), texture_atlases.add(bee_atlas));

        let bee_atlas = TextureAtlas::from_grid(
            texture_handles["bee_c-sheet"].clone(),
            Vec2::new(102., 104.),
            4,
            1,
        );
        texture_atlas_map.insert("bee_c".into(), texture_atlases.add(bee_atlas));

        let bee_atlas = TextureAtlas::from_grid(
            texture_handles["flower-sheet"].clone(),
            Vec2::new(102., 104.),
            17,
            1,
        );
        texture_atlas_map.insert("flower".into(), texture_atlases.add(bee_atlas));
    }
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST;
            }
        }
    }
}
