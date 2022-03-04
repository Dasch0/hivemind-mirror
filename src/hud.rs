use bevy::prelude::*;

use crate::{texture::TextureHandles, world::TimeStep};

pub enum HudUpdateEvent {
    //PlacementHud(Handle<Image>),
    FlowerMeter(u8),
    TreeMeter(u8),
    DeleteMeter(u8),
    SpeedChange(usize),
}

#[derive(Default)]
pub struct HudContext {
    pub flower_meter: MeterHud,
    pub tree_meter: MeterHud,
    pub delete_meter: MeterHud,
    pub hud_hovered: bool,
}

#[derive(Default)]
pub struct MeterHud {
    entity: Option<Entity>,
}

#[derive(Default, Component)]
pub struct TextHud {
    position: f32,
}

#[derive(Component, Clone, Copy)]
pub struct SpriteIndex(usize);

#[derive(Component)]
pub enum ClickAction {
    UpdateTimeStep(TimeStep),
}

pub fn setup(
    mut commands: Commands,
    texture_handles: Res<TextureHandles>,
    asset_server: Res<AssetServer>,
    mut hud_ctx: ResMut<HudContext>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    //let placement_hud = texture_handles["placement-hud"].clone();
    let speed_hud = texture_handles["speed-hud"].clone();
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // Ammo
            hud_ctx.flower_meter.entity = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(64.0), Val::Px(304.0)),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..Default::default()
                            },
                            //border: Rect::all(Val::Px(20.0)),
                            ..Default::default()
                        },
                        image: texture_handles["flower-meter1"].clone().into(),
                        ..Default::default()
                    })
                    .insert(Interaction::default())
                    .id(),
            );

            // Ammo
            hud_ctx.tree_meter.entity = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(64.0), Val::Px(304.0)),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(84.0),
                                bottom: Val::Px(10.0),
                                ..Default::default()
                            },
                            //border: Rect::all(Val::Px(20.0)),
                            ..Default::default()
                        },
                        image: texture_handles["tree-meter1"].clone().into(),
                        ..Default::default()
                    })
                    .insert(Interaction::default())
                    .id(),
            );

            // Ammo
            hud_ctx.delete_meter.entity = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(64.0), Val::Px(304.0)),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(158.0),
                                bottom: Val::Px(10.0),
                                ..Default::default()
                            },
                            //border: Rect::all(Val::Px(20.0)),
                            ..Default::default()
                        },
                        image: texture_handles["delete-meter1"].clone().into(),
                        ..Default::default()
                    })
                    .insert(Interaction::default())
                    .id(),
            );

            // Timer
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(400.0), Val::Px(100.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(10.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        //border: Rect::all(Val::Px(20.0)),
                        ..Default::default()
                    },
                    color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            size: Size::new(Val::Undefined, Val::Undefined),
                            margin: Rect {
                                left: Val::Auto,
                                right: Val::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        // Use the `Text::with_section` constructor
                        text: Text::with_section(
                            // Accepts a `String` or any type that converts into a `String`, such as `&str`
                            "00:00:00",
                            TextStyle {
                                font: asset_server.load("fonts/monogram.ttf"),
                                font_size: 75.0,
                                color: Color::WHITE,
                            },
                            // Note: You can use `Default::default()` in place of the `TextAlignment`
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    }).insert(crate::game::UsesTime);
                });

            // Speed Buttons
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(300.0), Val::Px(100.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Percent(1.),
                            right: Val::Percent(1.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    //color: Color::rgb(0.4, 0.4, 1.0).into(),
                    image: speed_hud.into(),
                    ..Default::default()
                })
                .insert(Interaction::default())
                .with_children(|parent| {
                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Percent(10.0),
                                    //bottom: Val::Percent(0.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            image: texture_handles["speed-btns2"].clone().into(),
                            ..Default::default()
                        })
                        .insert(SpriteIndex(1))
                        .insert(ClickAction::UpdateTimeStep(TimeStep::STOP));

                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Percent(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            image: texture_handles["speed-btns3"].clone().into(),
                            ..Default::default()
                        })
                        .insert(SpriteIndex(3))
                        .insert(ClickAction::UpdateTimeStep(TimeStep::PLAY));

                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(64.0), Val::Px(64.0)),

                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Percent(50.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            image: texture_handles["speed-btns5"].clone().into(),
                            ..Default::default()
                        })
                        .insert(SpriteIndex(5))
                        .insert(ClickAction::UpdateTimeStep(TimeStep::FAST));

                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Percent(70.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            image: texture_handles["speed-btns7"].clone().into(),
                            ..Default::default()
                        })
                        .insert(SpriteIndex(7))
                        .insert(ClickAction::UpdateTimeStep(TimeStep::FASTER));
                });
        });
}

pub fn hud_update_system(
    hud_ctx: ResMut<HudContext>,
    mut hud_events: EventReader<HudUpdateEvent>,
    texture_handles: Res<TextureHandles>,
    mut qs: QuerySet<(
        QueryState<&mut UiImage>,
        QueryState<(&SpriteIndex, &mut UiImage)>,
    )>,
) {
    for event in hud_events.iter() {
        match event {
            /*
            HudUpdateEvent::PlacementHud(image_handle) => {
                if let Some(placement) = hud_ctx.placement.entity {
                    if let Ok((mut image, mut style)) = query.get_mut(placement) {
                        info!(":)");
                        image.0 = image_handle.clone();
                        //style.size = Size::new(Val::Px(100.), Val::Px(100.));
                    }
                }
            }
            */
            HudUpdateEvent::FlowerMeter(count) => {
                if let Some(entity) = hud_ctx.flower_meter.entity {
                    if let Ok(mut image) = qs.q0().get_mut(entity) {
                        let key = format!("flower-meter{}", count + 1);
                        image.0 = texture_handles[&key].clone();
                    }
                }
            }
            HudUpdateEvent::TreeMeter(count) => {
                if let Some(entity) = hud_ctx.tree_meter.entity {
                    if let Ok(mut image) = qs.q0().get_mut(entity) {
                        let key = format!("tree-meter{}", count + 1);
                        info!("{}", key);
                        image.0 = texture_handles[&key].clone();
                    }
                }
            }
            HudUpdateEvent::DeleteMeter(count) => {
                if let Some(entity) = hud_ctx.delete_meter.entity {
                    if let Ok(mut image) = qs.q0().get_mut(entity) {
                        let key = format!("delete-meter{}", count + 1);
                        image.0 = texture_handles[&key].clone();
                    }
                }
            }

            HudUpdateEvent::SpeedChange(selected_idx) => {
                for (sprite_index, mut image) in qs.q1().iter_mut() {
                    let idx = if *selected_idx == sprite_index.0 {
                        selected_idx + 1
                    } else {
                        sprite_index.0
                    };
                    let key = format!("speed-btns{}", idx);
                    image.0 = texture_handles[&key].clone();
                }
            }
        }
    }
}

pub fn gatekeep_cursor_system(
    mut hud_ctx: ResMut<HudContext>,
    mut interaction_query: Query<&Interaction, Changed<Interaction>>,
) {
    let mut should_gatekeep = false;
    let mut lost_gatekeep = false;
    for interaction in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                should_gatekeep = true;
            }
            Interaction::Hovered => {
                should_gatekeep = true;
            }
            Interaction::None => {
                lost_gatekeep = true;
            }
        }
    }

    if lost_gatekeep || should_gatekeep {
        hud_ctx.hud_hovered = should_gatekeep;
    }
}

pub fn button_system(
    mut windows: ResMut<Windows>,
    mut time_step: ResMut<TimeStep>,
    mut hud_dispatcher: EventWriter<HudUpdateEvent>,

    query: Query<(&Interaction, &SpriteIndex, &ClickAction), Changed<Interaction>>,
) {
    let mut should_set_hover = false;
    let mut should_reset_cursor = false;
    for (interaction, sprite_index, action) in query.iter() {
        should_reset_cursor = true;
        match *interaction {
            Interaction::Clicked => {
                should_set_hover = true;

                match action {
                    ClickAction::UpdateTimeStep(ts) => time_step.set_from(ts),
                }

                hud_dispatcher.send(HudUpdateEvent::SpeedChange(sprite_index.0));
            }
            Interaction::Hovered => {
                should_set_hover = true;
            }
            Interaction::None => {}
        }
    }

    if should_reset_cursor || should_set_hover {
        let window = windows.get_primary_mut().unwrap();
        if should_set_hover {
            window.set_cursor_icon(CursorIcon::Hand);
        } else {
            window.set_cursor_icon(CursorIcon::Default);
        }
    }
}
