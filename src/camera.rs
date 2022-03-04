use crate::ui::{CursorEdge, UiContext};
use crate::RotationEvent;
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;
use bevy::{
    core::Time,
    input::mouse::{MouseButton, MouseMotion},
    input::{mouse::MouseWheel, Input},
    math::{Vec2Swizzles, Vec3},
    prelude::*,
    render::camera::Camera,
};
use bitflags::bitflags;

bitflags! {
    /// ALWAYS ^ 10
    /// IF & 10
    /// left = ^1 lower  bit
    /// right = keep lower bit
    /// IF !& 10
    /// left = keep lower  bit
    /// right = ^1 lower bit
    ///
    /// initial confifg 01
    /// go left => 11
    /// go right => 10
    /// go double => 00
    ///
    /// initial confifg 00
    /// go left => 10
    ///  go right -> 00
    /// go right => 11
    ///  go left -> 00
    /// go double => 01
    ///
    struct Orientation: u8 {
        const FLIP_AXIS = 0b01;
        const SWAP_AXIS = 0b10;
    }
}

#[derive(Default, Debug)]
pub struct CoordinateVectors {
    pub world_up: Vec3,
    pub world_right: Vec3,
    pub mouse_flip: Vec2,
    pub mouse_swap: bool,
    orientation: Orientation,
}

impl CoordinateVectors {
    pub fn new() -> Self {
        Self::default().with_orient()
    }

    pub fn with_orient(mut self) -> Self {
        self.orient();
        println!("[coord_vecs] {:#?}", self);
        self
    }

    pub fn rotate_right(&mut self) {
        self.orientation.rotate_right();
        self.orient();
    }

    pub fn rotate_left(&mut self) {
        self.orientation.rotate_left();
        self.orient();
    }

    pub fn orient(&mut self) {
        if self.orientation.bits & Orientation::SWAP_AXIS.bits != 0 {
            if self.orientation.bits & Orientation::FLIP_AXIS.bits == 0 {
                self.world_up.y = 0.;
                self.world_up.x = -1.;
                self.world_right.x = 0.;
                self.world_right.y = -1.;
                self.mouse_flip = Vec2::new(1.0, 1.0);
                self.mouse_swap = true;
            } else {
                self.world_up.y = 0.;
                self.world_up.x = 1.;
                self.world_right.x = 0.;
                self.world_right.y = 1.;
                self.mouse_swap = true;
                self.mouse_flip = Vec2::new(-1.0, -1.0);
            }
        } else if self.orientation.bits & Orientation::FLIP_AXIS.bits == 0 {
            self.world_up.x = 0.;
            self.world_up.y = 1.;
            self.world_right.y = 0.;
            self.world_right.x = -1.;
            self.mouse_flip = Vec2::new(1.0, -1.0);
            self.mouse_swap = false;
        } else {
            self.world_up.x = 0.;
            self.world_up.y = -1.;
            self.world_right.y = 0.;
            self.world_right.x = 1.;
            self.mouse_flip = Vec2::new(-1.0, 1.0);
            self.mouse_swap = false;
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::FLIP_AXIS
    }
}

impl Orientation {
    pub fn rotate_right(&mut self) {
        self.bits ^= Self::SWAP_AXIS.bits;
        if self.bits & Self::SWAP_AXIS.bits == 0 {
            self.bits ^= Self::FLIP_AXIS.bits;
        }
    }

    pub fn rotate_left(&mut self) {
        self.bits ^= Self::SWAP_AXIS.bits;
        if self.bits & Self::SWAP_AXIS.bits != 0 {
            self.bits ^= Self::FLIP_AXIS.bits;
        }
    }
}

// A simple camera system for moving and zooming the camera.
#[allow(clippy::too_many_arguments)]
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_scroll_event: EventReader<MouseWheel>,
    mut windows: ResMut<Windows>,
    mut ui_ctx: ResMut<UiContext>,
    mut rotation_events: EventWriter<RotationEvent>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Frustum>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut camera_updated = true;
        let window = windows.get_primary_mut().unwrap();

        if mouse_button_input.just_pressed(MouseButton::Other(0)) {
            window.set_cursor_visibility(false);
            ui_ctx.should_scroll = true;
        }

        if mouse_button_input.pressed(MouseButton::Other(0)) {
            for motion in mouse_motion_events.iter() {
                let cursor_flip = ui_ctx.coord_vecs.mouse_flip;
                let delta = ortho.scale * 0.2 * cursor_flip * motion.delta;
                direction += if ui_ctx.coord_vecs.mouse_swap {
                    delta.yx().extend(0.0)
                } else {
                    delta.extend(0.0)
                };
            }
        } else if mouse_button_input.just_released(MouseButton::Other(0)) {
            window.set_cursor_visibility(true);
            //window.set_cursor_position(ui_context.window_size / 2.0);
            ui_ctx.scroll_locked = false;
        }

        if !ui_ctx.scroll_locked {
            let mut should_scroll = false;
            for scroll in mouse_scroll_event.iter() {
                should_scroll = true;

                if scroll.y.abs() > 0.15 || ui_ctx.should_scroll {
                    ortho.scale += 0.1f32.copysign(-scroll.y);
                }
                camera_updated = true;
            }

            ui_ctx.should_scroll = should_scroll;
        }

        if keyboard_input.pressed(KeyCode::A) || ui_ctx.cursor_edge.map_or(false, CursorEdge::west)
        {
            direction -= ui_ctx.coord_vecs.world_right;
        }

        if keyboard_input.pressed(KeyCode::D) || ui_ctx.cursor_edge.map_or(false, CursorEdge::east)
        {
            direction += ui_ctx.coord_vecs.world_right;
        }

        if keyboard_input.pressed(KeyCode::W) || ui_ctx.cursor_edge.map_or(false, CursorEdge::north)
        {
            direction -= ui_ctx.coord_vecs.world_up;
        }

        if keyboard_input.pressed(KeyCode::S) || ui_ctx.cursor_edge.map_or(false, CursorEdge::south)
        {
            direction += ui_ctx.coord_vecs.world_up;
        }

        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.03;
            camera_updated = true;
        }

        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.03;
            camera_updated = true;
        }

        ortho.scale = ortho.scale.clamp(0.3, 2.5);

        if keyboard_input.just_pressed(KeyCode::Q) {
            let quat = Quat::from_rotation_z(-core::f32::consts::FRAC_PI_2);
            transform.rotation *= quat;
            ui_ctx.coord_vecs.rotate_left();

            camera_updated = true;
        }

        if keyboard_input.just_pressed(KeyCode::E) {
            let quat = Quat::from_rotation_z(core::f32::consts::FRAC_PI_2);
            transform.rotation *= quat;
            ui_ctx.coord_vecs.rotate_right();

            camera_updated = true;
        }

        if direction != Vec3::ZERO {
            camera_updated = true;
        }
        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * 500. * ortho.scale;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;

        if camera_updated {
            rotation_events.send(RotationEvent);
        }
    }
}
