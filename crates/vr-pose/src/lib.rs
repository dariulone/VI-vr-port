#![forbid(unsafe_code)]

pub const POSE_RECORD_STRIDE: usize = 0x80;
pub const POSE_METADATA_FLAG_OFFSET: usize = 0x63;
pub const POSE_AUX_BLOCK_COUNT: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoseRecord128 {
    pub bytes: [u8; POSE_RECORD_STRIDE],
}

impl PoseRecord128 {
    pub fn metadata_flag(self) -> u8 {
        self.bytes[POSE_METADATA_FLAG_OFFSET]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoseBuffer {
    pub records: Vec<PoseRecord128>,
}

impl PoseBuffer {
    pub fn new(count: usize) -> Self {
        Self {
            records: vec![PoseRecord128 { bytes: [0; POSE_RECORD_STRIDE] }; count],
        }
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn replace_record(&mut self, index: usize, record: PoseRecord128) -> bool {
        if let Some(slot) = self.records.get_mut(index) {
            *slot = record;
            true
        } else {
            false
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoBoneTarget {
    pub shoulder: [f32; 3],
    pub hand: [f32; 3],
    pub elbow_hint: [f32; 3],
}

pub fn blend_weight(in_vr: bool, cutscene: bool, ragdoll: bool) -> f32 {
    if in_vr && !cutscene && !ragdoll {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pose_records_keep_the_static_stride() {
        let buffer = PoseBuffer::new(596);
        assert_eq!(buffer.count(), 596);
        assert_eq!(std::mem::size_of::<PoseRecord128>(), POSE_RECORD_STRIDE);
        assert_eq!(POSE_AUX_BLOCK_COUNT, 4);
    }

    #[test]
    fn pose_metadata_flag_uses_the_static_byte_offset() {
        let mut record = PoseRecord128 { bytes: [0; POSE_RECORD_STRIDE] };
        record.bytes[POSE_METADATA_FLAG_OFFSET] = 7;
        assert_eq!(record.metadata_flag(), 7);
    }

    #[test]
    fn vrik_blend_disables_for_cutscene_or_ragdoll() {
        assert_eq!(blend_weight(true, false, false), 1.0);
        assert_eq!(blend_weight(true, true, false), 0.0);
        assert_eq!(blend_weight(true, false, true), 0.0);
    }
}

