#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eye {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyeFov {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyePacket {
    pub frame_id: u64,
    pub eye: Eye,
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub camera_world: [f32; 3],
    pub near_z: f32,
    pub far_z: f32,
    pub viewport: [u32; 4],
    pub image_handle: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrustumBounds {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

impl FrustumBounds {
    pub fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            up: self.up.max(other.up),
            down: self.down.min(other.down),
        }
    }
}

pub fn projection_from_tangents(fov: EyeFov, near_z: f32, far_z: f32) -> [[f32; 4]; 4] {
    let width = fov.right - fov.left;
    let height = fov.up - fov.down;
    let depth = far_z - near_z;

    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, 2.0 / height, 0.0, 0.0],
        [
            (fov.right + fov.left) / width,
            (fov.up + fov.down) / height,
            far_z / depth,
            1.0,
        ],
        [0.0, 0.0, -(near_z * far_z) / depth, 0.0],
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AimMode {
    Camera,
    Muzzle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimPacket {
    pub frame_id: u64,
    pub mode: AimMode,
    pub grip_world: [f32; 3],
    pub muzzle_world: [f32; 3],
    pub muzzle_forward: [f32; 3],
    pub trigger_pressed: bool,
}

impl AimPacket {
    pub fn shot_origin(self) -> [f32; 3] {
        match self.mode {
            AimMode::Camera => self.grip_world,
            AimMode::Muzzle => self.muzzle_world,
        }
    }

    pub fn shot_direction(self) -> [f32; 3] {
        match self.mode {
            AimMode::Camera => normalize(self.grip_world),
            AimMode::Muzzle => normalize(self.muzzle_forward),
        }
    }
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if length <= f32::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    [value[0] / length, value[1] / length, value[2] / length]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asymmetric_projection_preserves_optical_center() {
        let matrix = projection_from_tangents(
            EyeFov {
                left: -1.0,
                right: 1.2,
                up: 1.0,
                down: -1.0,
            },
            0.1,
            1000.0,
        );
        assert!(matrix[2][0] > 0.0);
        assert_eq!(matrix[0][1], 0.0);
    }

    #[test]
    fn union_bounds_cover_both_eyes() {
        let left = FrustumBounds {
            left: -1.0,
            right: 0.8,
            up: 1.0,
            down: -0.9,
        };
        let right = FrustumBounds {
            left: -0.8,
            right: 1.1,
            up: 1.1,
            down: -1.0,
        };
        let union = left.union(right);
        assert_eq!(union.left, -1.0);
        assert_eq!(union.right, 1.1);
        assert_eq!(union.up, 1.1);
        assert_eq!(union.down, -1.0);
    }

    #[test]
    fn muzzle_mode_decouples_shot_origin_from_grip() {
        let packet = AimPacket {
            frame_id: 7,
            mode: AimMode::Muzzle,
            grip_world: [0.0, 0.0, 0.0],
            muzzle_world: [1.0, 2.0, 3.0],
            muzzle_forward: [0.0, 0.0, 2.0],
            trigger_pressed: true,
        };
        assert_eq!(packet.shot_origin(), [1.0, 2.0, 3.0]);
        assert_eq!(packet.shot_direction(), [0.0, 0.0, 1.0]);
    }
}

