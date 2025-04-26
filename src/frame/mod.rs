pub mod audio;
pub(crate) mod drop_guard;
pub mod generic;
pub mod metadata;
pub mod video;

use crate::bindings;

pub use generic::NDIFrame;

#[allow(private_bounds)]
pub trait RawFrame: RawFrameInner {}

pub(crate) trait RawFrameInner {
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t);
    fn assert_unwritten(&self);
}

pub trait NDIFrameExt<T: RawFrame> {
    fn data_valid(&self) -> bool;

    /// Frees the buffers
    /// Note: Other properties like resolution, frame rate and FourCC might or might not be retained
    fn clear(&mut self);
}
