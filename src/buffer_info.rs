//! Contains information about the memory layout of video frame buffers

use crate::{enums::NDIFieldedFrameMode, resolution::Resolution, subsampling::Subsampling};

/// Contains information about the memory layout of video frame buffers
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferInfo {
    /// The size of the buffer in bytes. Takes into account the field mode.
    pub size: usize,
    /// The stride/size of a single line in bytes.
    pub line_stride: usize,

    pub resolution: Resolution,
    /// The field mode of the frame (progressive or interlaced).
    pub field_mode: NDIFieldedFrameMode,
    /// Information about chroma subsampling.
    pub subsampling: Subsampling,
}
