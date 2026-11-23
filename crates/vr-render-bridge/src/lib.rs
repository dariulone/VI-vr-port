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
    pub frustum_range_process: usize,
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
    pub vehicle_request_system: usize,
    pub vehicle_deform_init: usize,
    pub vehicle_deform_layout_a: usize,
    pub vehicle_deform_layout_b: usize,
    pub explosion_damage_init: usize,
    pub gpu_particle_collision_init: usize,
    pub rain_collision_init: usize,
    pub rain_collision_resource: usize,
    pub rain_persistent_resource: usize,
    pub water_simulation_init: usize,
    pub water_system_state: usize,
    pub water_state_builder: usize,
    pub water_system_helper: usize,
    pub constraint_name_resolver: usize,
    pub horse_collision_response: usize,
    pub physics_material_params: usize,
    pub rigid_body_params: usize,
    pub rigid_body_config: usize,
    pub rigid_body_dynamic_update: usize,
    pub body_shape_batch_update: usize,
    pub body_weighted_aggregate: usize,
    pub body_second_moment_accumulator: usize,
    pub body_transform_normalizer: usize,
    pub body_inverse_mass_recompute: usize,
    pub body_shape_property_update: usize,
    pub body_weighted_center: usize,
    pub body_inertia_accumulator: usize,
    pub body_property_initializer: usize,
    pub vehicle_handling_data: usize,
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
    frustum_range_process: 0x2DAB0,
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
    vehicle_request_system: 0x5814350,
    vehicle_deform_init: 0xCBF1F0,
    vehicle_deform_layout_a: 0x5B02F40,
    vehicle_deform_layout_b: 0x5432060,
    explosion_damage_init: 0x2511AB0,
    gpu_particle_collision_init: 0x1AF3BA0,
    rain_collision_init: 0x1D87EF0,
    rain_collision_resource: 0x5D44230,
    rain_persistent_resource: 0x5C0C5F0,
    water_simulation_init: 0x1E490F0,
    water_system_state: 0x26205C0,
    water_state_builder: 0x5B83070,
    water_system_helper: 0x2620140,
    constraint_name_resolver: 0x26870F0,
    horse_collision_response: 0x1D8C9B0,
    physics_material_params: 0x4896800,
    rigid_body_params: 0x4454BE0,
    rigid_body_config: 0x4455000,
    rigid_body_dynamic_update: 0x1F97990,
    body_shape_batch_update: 0x645740,
    body_weighted_aggregate: 0x4F5B770,
    body_second_moment_accumulator: 0x4F5B8C0,
    body_transform_normalizer: 0x517E960,
    body_inverse_mass_recompute: 0x53808B0,
    body_shape_property_update: 0x2687A20,
    body_weighted_center: 0x55B1160,
    body_inertia_accumulator: 0x55B1200,
    body_property_initializer: 0x6D9FF0,
    vehicle_handling_data: 0x56D8090,
};

pub const QUEUE_PLANE_SUBMIT_SLOT: usize = 0x100;
pub const QUEUE_BATCH_SUBMIT_SLOT: usize = 0x118;
pub const EXPECTED_IMAGE_BASE: usize = 0x140000000;
pub const EXPECTED_INPUT_SHA256: &str =
    "E95020B2FA4A721DB3DF59FDECF75621730AC559D8835252F6C3B658D5A08993";

pub fn manifest_matches(image_base: usize, input_sha256: &str) -> bool {
    image_base == EXPECTED_IMAGE_BASE && input_sha256.eq_ignore_ascii_case(EXPECTED_INPUT_SHA256)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookTarget {
    DeviceInit = 0,
    CameraOwnerTick = 1,
    CameraRenderPrepare = 2,
    CameraPacketCopy = 3,
    FrustumPrepare = 4,
    FrustumShapeMath = 5,
    FrustumResourceDispatch = 6,
    FrustumRangeDispatch = 7,
    FrustumRangeProcess = 8,
    InstanceCullingDispatch = 9,
    Worker = 10,
    PresentCore = 11,
    PresentLifecycle = 12,
    PoseCopy = 13,
    PoseFinalize = 14,
    BoneHashMap = 15,
}

pub const LIVE_HOOK_ORDER: [HookTarget; 16] = [
    HookTarget::DeviceInit,
    HookTarget::CameraOwnerTick,
    HookTarget::CameraRenderPrepare,
    HookTarget::CameraPacketCopy,
    HookTarget::FrustumPrepare,
    HookTarget::FrustumShapeMath,
    HookTarget::FrustumResourceDispatch,
    HookTarget::FrustumRangeDispatch,
    HookTarget::FrustumRangeProcess,
    HookTarget::InstanceCullingDispatch,
    HookTarget::Worker,
    HookTarget::PresentCore,
    HookTarget::PresentLifecycle,
    HookTarget::PoseCopy,
    HookTarget::PoseFinalize,
    HookTarget::BoneHashMap,
];

impl HookTarget {
    pub const fn rva(self, manifest: RvaManifest) -> usize {
        match self {
            Self::DeviceInit => manifest.device_init,
            Self::CameraOwnerTick => manifest.camera_owner_tick,
            Self::CameraRenderPrepare => manifest.camera_render_prepare,
            Self::CameraPacketCopy => manifest.camera_packet_copy,
            Self::FrustumPrepare => manifest.camera_frustum_prepare,
            Self::FrustumShapeMath => manifest.camera_frustum_shape_math,
            Self::FrustumResourceDispatch => manifest.frustum_resource_dispatch,
            Self::FrustumRangeDispatch => manifest.frustum_range_dispatch,
            Self::FrustumRangeProcess => manifest.frustum_range_process,
            Self::InstanceCullingDispatch => manifest.instance_culling_dispatch,
            Self::Worker => manifest.worker,
            Self::PresentCore => manifest.present_core,
            Self::PresentLifecycle => manifest.present_lifecycle,
            Self::PoseCopy => manifest.pose_copy,
            Self::PoseFinalize => manifest.pose_finalize,
            Self::BoneHashMap => manifest.bone_hash_map,
        }
    }
}

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
        assert_ne!(CURRENT_MANIFEST.frustum_range_process, 0);
        assert_ne!(CURRENT_MANIFEST.frustum_resource_wrapper, 0);
        assert_ne!(CURRENT_MANIFEST.frustum_intersection_math, 0);
        assert_ne!(CURRENT_MANIFEST.pose_gate_update, 0);
        assert_eq!(QUEUE_PLANE_SUBMIT_SLOT, 0x100);
        assert_eq!(QUEUE_BATCH_SUBMIT_SLOT, 0x118);
        assert!(manifest_matches(
            EXPECTED_IMAGE_BASE,
            EXPECTED_INPUT_SHA256
        ));
        assert!(!manifest_matches(EXPECTED_IMAGE_BASE + 0x1000, EXPECTED_INPUT_SHA256));
        assert_ne!(CURRENT_MANIFEST.pose_finalize, 0);
        assert_ne!(CURRENT_MANIFEST.pose_aux_array_build, 0);
        assert_ne!(CURRENT_MANIFEST.projectile_location, 0);
        assert_ne!(CURRENT_MANIFEST.vehicle_deform_init, 0);
        assert_ne!(CURRENT_MANIFEST.vehicle_deform_layout_a, 0);
        assert_ne!(CURRENT_MANIFEST.explosion_damage_init, 0);
        assert_ne!(CURRENT_MANIFEST.gpu_particle_collision_init, 0);
        assert_ne!(CURRENT_MANIFEST.rain_collision_resource, 0);
        assert_ne!(CURRENT_MANIFEST.water_system_state, 0);
        assert_ne!(CURRENT_MANIFEST.physics_material_params, 0);
        assert_ne!(CURRENT_MANIFEST.rigid_body_params, 0);
        assert_ne!(CURRENT_MANIFEST.rigid_body_dynamic_update, 0);
        assert_ne!(CURRENT_MANIFEST.body_shape_batch_update, 0);
        assert_ne!(CURRENT_MANIFEST.body_weighted_aggregate, 0);
        assert_ne!(CURRENT_MANIFEST.body_second_moment_accumulator, 0);
        assert_ne!(CURRENT_MANIFEST.body_transform_normalizer, 0);
        assert_ne!(CURRENT_MANIFEST.body_inverse_mass_recompute, 0);
        assert_ne!(CURRENT_MANIFEST.body_shape_property_update, 0);
        assert_ne!(CURRENT_MANIFEST.body_weighted_center, 0);
        assert_ne!(CURRENT_MANIFEST.body_inertia_accumulator, 0);
        assert_ne!(CURRENT_MANIFEST.body_property_initializer, 0);
        assert_eq!(LIVE_HOOK_ORDER.len(), 16);
        assert_eq!(LIVE_HOOK_ORDER[0].rva(CURRENT_MANIFEST), CURRENT_MANIFEST.device_init);
        assert_eq!(LIVE_HOOK_ORDER[15].rva(CURRENT_MANIFEST), CURRENT_MANIFEST.bone_hash_map);
        assert_eq!(present_path(), PresentPath::XboxPlaneSubmission);
    }
}

