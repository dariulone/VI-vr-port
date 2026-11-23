#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RvaManifest {
    pub device_init: usize,
    pub worker: usize,
    pub present_lifecycle: usize,
    pub present_core: usize,
    pub camera_output: usize,
    pub pose_copy: usize,
    pub bone_hash_map: usize,
}

pub const CURRENT_MANIFEST: RvaManifest = RvaManifest {
    device_init: 0x2478440,
    worker: 0x2481E90,
    present_lifecycle: 0x2484180,
    present_core: 0x24826A0,
    camera_output: 0x322F8D0,
    pose_copy: 0x523B680,
    bone_hash_map: 0x523C6F0,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentPath {
    Unknown,
    XboxPlaneSubmission,
}

pub fn present_path() -> PresentPath {
    PresentPath::XboxPlaneSubmission
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_contains_the_native_stereo_boundaries() {
        assert_ne!(CURRENT_MANIFEST.present_lifecycle, 0);
        assert_ne!(CURRENT_MANIFEST.present_core, 0);
        assert_eq!(present_path(), PresentPath::XboxPlaneSubmission);
    }
}

