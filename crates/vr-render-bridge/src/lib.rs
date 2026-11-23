#![forbid(unsafe_code)]

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RvaManifest {
    pub device_init: usize,
    pub frame_begin: usize,
    pub worker: usize,
    pub queue_execute_or_sync: usize,
    pub present_lifecycle: usize,
    pub present_core: usize,
    pub camera_output_builder: usize,
    pub camera_output_finalizer: usize,
    pub camera_view_update: usize,
    pub camera_clip_update: usize,
    pub camera_owner_tick: usize,
    pub camera_render_prepare: usize,
    pub camera_packet_copy: usize,
    pub camera_render_object_init: usize,
    pub camera_render_object_accessor: usize,
    pub render_resource_register: usize,
    pub render_resource_finalize: usize,
    pub camera_visibility_consumer: usize,
    pub camera_visibility_handle: usize,
    pub camera_frustum_prepare: usize,
    pub camera_frustum_shape_math: usize,
    pub frustum_resource_wrapper: usize,
    pub frustum_resource_dispatch: usize,
    pub frustum_range_dispatch: usize,
    pub frustum_intersection_math: usize,
    pub instance_culling_init: usize,
    pub instance_culling_dispatch: usize,
    pub culling_range_accumulator: usize,
    pub meshlet_culling_init: usize,
    pub pose_source: usize,
    pub pose_copy: usize,
    pub pose_finalize: usize,
    pub pose_aux_array_build: usize,
    pub pose_gate_update: usize,
    pub bone_hash_map: usize,
    pub skinning_record_builder: usize,
    pub precision_context: usize,
    pub precision_context_update: usize,
    pub weapon_components: usize,
    pub projectile_location: usize,
}

pub const CURRENT_MANIFEST: RvaManifest = RvaManifest {
    device_init: 0x2478440,
    frame_begin: 0x41E9E0,
    worker: 0x2481E90,
    queue_execute_or_sync: 0x2482660,
    present_lifecycle: 0x2484180,
    present_core: 0x24826A0,
    camera_output_builder: 0x322F5D0,
    camera_output_finalizer: 0x322F8D0,
    camera_view_update: 0x3230750,
    camera_clip_update: 0x32307F0,
    camera_owner_tick: 0x2008AE0,
    camera_render_prepare: 0x3231B30,
    camera_packet_copy: 0x53159B0,
    camera_render_object_init: 0x5CA410,
    camera_render_object_accessor: 0x4FEBB80,
    render_resource_register: 0xD0320,
    render_resource_finalize: 0xD0610,
    camera_visibility_consumer: 0x31B0B0,
    camera_visibility_handle: 0x31BC70,
    camera_frustum_prepare: 0x4128B0,
    camera_frustum_shape_math: 0x4440C60,
    frustum_resource_wrapper: 0x559A400,
    frustum_resource_dispatch: 0x4E29640,
    frustum_range_dispatch: 0x4E296B0,
    frustum_intersection_math: 0x4E298A0,
    instance_culling_init: 0x5700770,
    instance_culling_dispatch: 0x25D8590,
    culling_range_accumulator: 0x55BFC50,
    meshlet_culling_init: 0x5B03CB0,
    pose_source: 0x523B630,
    pose_copy: 0x523B680,
    pose_finalize: 0x523BD00,
    pose_aux_array_build: 0x523C690,
    pose_gate_update: 0x523C810,
    bone_hash_map: 0x523C6F0,
    skinning_record_builder: 0x27C15A0,
    precision_context: 0xF1B130,
    precision_context_update: 0x4ACB720,
    weapon_components: 0x1D1BBF0,
    projectile_location: 0x5B20870,
};

pub const QUEUE_PLANE_SUBMIT_SLOT: usize = 0x100;
pub const QUEUE_BATCH_SUBMIT_SLOT: usize = 0x118;

#[repr(u8)]
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
        assert_ne!(CURRENT_MANIFEST.camera_output_finalizer, 0);
        assert_ne!(CURRENT_MANIFEST.camera_packet_copy, 0);
        assert_ne!(CURRENT_MANIFEST.instance_culling_dispatch, 0);
        assert_ne!(CURRENT_MANIFEST.camera_visibility_consumer, 0);
        assert_ne!(CURRENT_MANIFEST.camera_frustum_prepare, 0);
        assert_ne!(CURRENT_MANIFEST.frustum_resource_dispatch, 0);
        assert_ne!(CURRENT_MANIFEST.frustum_range_dispatch, 0);
        assert_eq!(QUEUE_PLANE_SUBMIT_SLOT, 0x100);
        assert_eq!(QUEUE_BATCH_SUBMIT_SLOT, 0x118);
        assert_ne!(CURRENT_MANIFEST.pose_finalize, 0);
        assert_ne!(CURRENT_MANIFEST.pose_aux_array_build, 0);
        assert_ne!(CURRENT_MANIFEST.projectile_location, 0);
        assert_eq!(present_path(), PresentPath::XboxPlaneSubmission);
    }
}

