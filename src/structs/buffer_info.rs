use crate::{
    enums::NDIFieldedFrameMode,
    structs::{resolution::Resolution, subsampling::Subsampling},
};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferInfo {
    pub size: usize,
    pub line_stride: usize,
    pub resolution: Resolution,
    pub field_mode: NDIFieldedFrameMode,
    pub subsampling: Subsampling,
}
