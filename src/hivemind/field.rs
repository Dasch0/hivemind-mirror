/// Global data for hivemind ai's, regardless of colony
// TODO: Top level todos
//  * Figure out how to scale field values as a factor of world size. This assumes we care about
//     variable world sizes and also that the hivemind parameters need to change based on world
//     size
use bevy::prelude::*;
use bevy::math;
use std::f32::consts;

/// LUT of direction vectors to neighboring grids
const DIRS: [IVec2; 9] = [
    math::const_ivec2!([-1, -1]),
    math::const_ivec2!([0, -1]),
    math::const_ivec2!([1, -1]),
    math::const_ivec2!([-1, -0]),
    math::const_ivec2!([0, 0]),
    math::const_ivec2!([1, 0]),
    math::const_ivec2!([-1, 1]),
    math::const_ivec2!([0, 1]),
    math::const_ivec2!([1, 1]),
];

/// Data representing a field of vectors able to be directly updated with some constant decay
// TODO: Implemented via dual linear buffers, quadtree probably faster
#[derive(Component)]
pub struct Vector<const W: usize, const H: usize> {
    /// 2 2d arrays of vectors for dual buffer updating. The first index is active_buffer
    data: [[[Vec2; W]; H]; 2],
    pub decay: f32,
    pub max: f32,
    pub lerp_coef: f32,
    pub update_coef: f32,
    // stored as a bool for easy negation
    active_buffer: bool,
}

impl<const W: usize, const H: usize> Vector<W, H> {
    pub const DECAY: f32 = 1.0;
    pub const MAX: f32 = 10.0;
    pub const LERP_COEF: f32 = 0.9; // should be between 0.0 and 1.0
    pub const UPDATE_COEF: f32 = 0.5;

    /// Create a new vector field.
    ///
    /// # Arguments
    /// * `decay` - The time constant of the exponential decay of the field strength. Larger values lead to slower decay
    /// * `max` - The maximum allowed magnitude of any vector in the field
    /// * `lerp_coef` - a 0.0..1.0 value biasing towards diffusion at smaller values, and adjection
    /// at larger values
    /// * `update_coef` - Modifies the simulation time step to speed up or slow down field
    /// diffusion/adjection
    pub fn new(decay: f32, max: f32, lerp_coef: f32, update_coef: f32) -> Self {
        let decay = f32::abs(decay);
        Self {
            data: [[[Vec2::ZERO; W]; H]; 2],
            decay,
            max,
            lerp_coef,
            update_coef,
            active_buffer: false,
        }
    }

    pub fn default() -> Self {
        Self::new(Self::DECAY, Self::MAX, Self::LERP_COEF, Self::UPDATE_COEF)
    }

    /// Simulate diffusion and decay of vector field
    pub fn update(&mut self, time_step: f32) {
        // Implementation note:
        //  This attempts to approximate vector diffusion without doing a full GVT/fluid
        //  model.
        //  1. Component-wise diffusion is performed with a simple box blur filter
        //  (average of all neighbors).
        //  2. Adjection is approximated by pointing directly at the neighboring
        //     cell with the strongest nearby vector. The resulting vector uses the neighbor's
        //     magntitude
        //  3. The diffusion and adjection vectors are interpolated together with a paramaterized
        //     coefficient (0.0 to 1.0, smaller values bias towards diffusion)
        //  4. The resulting vector is exponentially decayed towards zero, to avoid unbounded
        //     energy gain into the system
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                // per element
                let loc = UVec2::new(x as u32, y as u32);
                let mut max_dir = self[loc].normalize_or_zero();
                let mut max = self[loc].length();
                let mut sum = Vec2::ZERO;

                for dir in DIRS {
                    let val = self[(loc.as_ivec2() + dir).as_uvec2()];
                    sum += val;

                    if max < val.length() {
                        max = val.length();
                        max_dir = dir.as_vec2();
                    }
                }
                let diffusion = sum / 9.0; // component-wise diffused vector
                let adjection = max * max_dir; // adjected vector approximation, points at the strongest neighbor vec
                let decay = consts::E.powf(-time_step / self.decay);
                let delta = diffusion.lerp(adjection, 0.1) - self[loc];

                self.data[!self.active_buffer as usize][y][x] =
                    self[loc] + delta * self.update_coef * time_step;
                self.data[!self.active_buffer as usize][y][x] *= decay; // apply decay after update since it already uses timestep
            }
        }
        self.active_buffer = !self.active_buffer;
    }
}

impl<const W: usize, const H: usize> std::ops::Index<usize> for Vector<W, H> {
    type Output = [Vec2];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.active_buffer as usize][index]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<Vec2> for Vector<W, H> {
    type Output = Vec2;

    fn index(&self, index: Vec2) -> &Self::Output {
        &self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<usize> for Vector<W, H> {
    fn index_mut(&mut self, index: usize) -> &mut [Vec2] {
        &mut self.data[self.active_buffer as usize][index]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<Vec2> for Vector<W, H> {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        &mut self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<UVec2> for Vector<W, H> {
    type Output = Vec2;

    fn index(&self, index: UVec2) -> &Self::Output {
        &self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<UVec2> for Vector<W, H> {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        &mut self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

/// Data representing a diffusing field with varying strength. Models gradients, sources, and sinks
#[derive(Component)]
pub struct Scalar<const W: usize, const H: usize> {
    /// 2 2d arrays of vectors for dual buffer updating. The first index is active_buffer
    data: [[[f32; W]; H]; 2],
    pub min: f32,
    pub max: f32,
    pub decay: f32,
    pub update_coef: f32,
    // stored as a bool for easy negation
    active_buffer: bool,
}

impl<const W: usize, const H: usize> Scalar<W, H> {
    pub const MIN: f32 = 0.1;
    pub const MAX: f32 = 100.0;
    pub const DECAY: f32 = 1.0;
    pub const UPDATE_COEF: f32 = 10.0;

    pub fn default() -> Self {
        Self::new(Self::MIN, Self::MAX, Self::DECAY, Self::UPDATE_COEF)
    }

    pub fn default_wall() -> Self {
        Self::new(Self::MIN, Self::MAX / 10., 0.5, 0.005) // wall fields should have a very small radius, so fast decay and low diffusion rate
    }

    /// Create a new diffuse field
    ///
    /// # Arguments
    /// * `min` - smallest allowed field value, usually should be >= 0.0
    /// * `max` - largest allowed field value
    /// * `decay` - the time constant of the exponential decay function. Larger values lead to
    /// slower decay
    /// * `update_coef` - Modifies the simulation time step to speed up or slow down field
    pub fn new(min: f32, max: f32, decay: f32, update_coef: f32) -> Self {
        let decay = f32::abs(decay);
        Self {
            data: [[[0.0; W]; H]; 2],
            min,
            max,
            decay,
            update_coef,
            active_buffer: false,
        }
    }

    /// Update the field simulation. Performing diffusion and recalculating gradients
    /// Implementation performs:
    ///     1. Diffuse using a 3x3 box filter
    ///     2. Randomized exponential decay towards min value
    ///     3. Gradient recalcuation using a 3x3 kernel with weights [-0.5, 0, 0.5] for each axis
    pub fn update(&mut self, time_step: f32) {
        for y in 1..H - 1 {
            for x in 1..W - 1 {
                // per element
                let loc = UVec2::new(x as u32, y as u32);
                let mut sum = 0.0;
                for dir in DIRS.iter() {
                    let val = self[(loc.as_ivec2() + *dir).as_uvec2()];
                    sum += val;
                }
                let diffusion = sum / 9.0;
                let decay = consts::E.powf(-time_step / self.decay);

                self.data[!self.active_buffer as usize][y][x] =
                    self[y][x] + (diffusion - self[y][x]) * time_step;
                self.data[!self.active_buffer as usize][y][x] *= decay;

                self.data[!self.active_buffer as usize][y][x] = f32::clamp(
                    self.data[!self.active_buffer as usize][y][x],
                    self.min,
                    self.max,
                );
                // apply decay after update since decay is already dependent on timestep
                //self.data[!self.active_buffer as usize][y][x] *= decay;
            }
        }
        self.active_buffer = !self.active_buffer;
    }

    /// Compute scalar field gradient at position
    pub fn grad(&self, pos: Vec2) -> Vec2 {
        //FIXME: replace with proper bounds checking
        if (pos.x < 0.) || ((pos.x as usize) > W - 1) {
            Vec2::ZERO
        } else if (pos.y < 0.) || ((pos.y as usize) > H - 1) {
            Vec2::ZERO
        } else {
            let x1 = self[pos - Vec2::X];
            let x2 = self[pos + Vec2::X];
            let y1 = self[pos - Vec2::Y];
            let y2 = self[pos + Vec2::Y];

            let dx = -0.5 * x1 + 0.5 * x2;
            let dy = -0.5 * y1 + 0.5 * y2;

            Vec2::new(dx, dy)
        }
    }
}

impl<const W: usize, const H: usize> std::ops::Index<usize> for Scalar<W, H> {
    type Output = [f32];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.active_buffer as usize][index]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<Vec2> for Scalar<W, H> {
    type Output = f32;

    fn index(&self, index: Vec2) -> &Self::Output {
        &self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<usize> for Scalar<W, H> {
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        &mut self.data[self.active_buffer as usize][index]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<Vec2> for Scalar<W, H> {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        &mut self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::Index<UVec2> for Scalar<W, H> {
    type Output = f32;

    fn index(&self, index: UVec2) -> &Self::Output {
        &self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}

impl<const W: usize, const H: usize> std::ops::IndexMut<UVec2> for Scalar<W, H> {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        &mut self.data[self.active_buffer as usize][index.y as usize][index.x as usize]
    }
}
