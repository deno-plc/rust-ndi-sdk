//! Handling and manipulation of video/audio/metadata frames

pub mod audio;
pub(crate) mod drop_guard;
pub mod generic;
pub mod metadata;
pub mod video;

use crate::frame::drop_guard::RawBufferManagement;

pub use generic::NDIFrame;

/// Trait for inner raw frames, therefore it has no publicly visible implementors
#[allow(private_bounds)]
pub trait RawFrame: RawBufferManagement + Send + Sync {}
