#![forbid(unsafe_code)]

use gta6_vr_core::{Eye, EyePacket};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameStage {
    Idle = 0,
    AcquiringViews = 1,
    RenderingLeft = 2,
    RenderingRight = 3,
    ReleasingImages = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    WrongStage,
    WrongEye,
    FrameMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameCoordinator {
    frame_id: u64,
    stage: FrameStage,
}

impl FrameCoordinator {
    pub const fn new() -> Self {
        Self {
            frame_id: 0,
            stage: FrameStage::Idle,
        }
    }

    pub const fn stage(self) -> FrameStage {
        self.stage
    }

    pub const fn frame_id(self) -> u64 {
        self.frame_id
    }

    pub fn begin_frame(&mut self, frame_id: u64) -> Result<(), FrameError> {
        if self.stage != FrameStage::Idle {
            return Err(FrameError::WrongStage);
        }
        self.frame_id = frame_id;
        self.stage = FrameStage::AcquiringViews;
        Ok(())
    }

    pub fn publish_views(&mut self, frame_id: u64) -> Result<(), FrameError> {
        if self.stage != FrameStage::AcquiringViews {
            return Err(FrameError::WrongStage);
        }
        if self.frame_id != frame_id {
            return Err(FrameError::FrameMismatch);
        }
        self.stage = FrameStage::RenderingLeft;
        Ok(())
    }

    pub fn begin_eye(&mut self, eye: Eye) -> Result<(), FrameError> {
        match (self.stage, eye) {
            (FrameStage::RenderingLeft, Eye::Left)
            | (FrameStage::RenderingRight, Eye::Right) => Ok(()),
            _ => Err(FrameError::WrongEye),
        }
    }

    pub fn finish_eye(&mut self, eye: Eye) -> Result<(), FrameError> {
        match (self.stage, eye) {
            (FrameStage::RenderingLeft, Eye::Left) => {
                self.stage = FrameStage::RenderingRight;
                Ok(())
            }
            (FrameStage::RenderingRight, Eye::Right) => {
                self.stage = FrameStage::ReleasingImages;
                Ok(())
            }
            _ => Err(FrameError::WrongEye),
        }
    }

    pub fn submit(&mut self, frame_id: u64) -> Result<(), FrameError> {
        if self.stage != FrameStage::ReleasingImages {
            return Err(FrameError::WrongStage);
        }
        if self.frame_id != frame_id {
            return Err(FrameError::FrameMismatch);
        }
        self.stage = FrameStage::Idle;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.stage = FrameStage::Idle;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StereoViews {
    pub left: EyePacket,
    pub right: EyePacket,
}

impl StereoViews {
    pub fn frame_id(self) -> Option<u64> {
        (self.left.frame_id == self.right.frame_id).then_some(self.left.frame_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinator_requires_both_eyes_before_submit() {
        let mut coordinator = FrameCoordinator::new();
        coordinator.begin_frame(7).unwrap();
        coordinator.publish_views(7).unwrap();
        coordinator.begin_eye(Eye::Left).unwrap();
        coordinator.finish_eye(Eye::Left).unwrap();
        coordinator.begin_eye(Eye::Right).unwrap();
        coordinator.finish_eye(Eye::Right).unwrap();
        coordinator.submit(7).unwrap();
        assert_eq!(coordinator.stage(), FrameStage::Idle);
    }

    #[test]
    fn coordinator_rejects_frame_mismatch() {
        let mut coordinator = FrameCoordinator::new();
        coordinator.begin_frame(7).unwrap();
        assert_eq!(coordinator.publish_views(8), Err(FrameError::FrameMismatch));
        coordinator.abort();
        assert_eq!(coordinator.stage(), FrameStage::Idle);
    }
}
