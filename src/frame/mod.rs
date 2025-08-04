pub mod audio;
pub(crate) mod drop_guard;
pub mod generic;
pub mod metadata;
pub mod video;

use crate::bindings;

pub use generic::NDIFrame;

#[allow(private_bounds)]
pub trait RawFrame: RawBufferManagement {}

pub(crate) trait RawBufferManagement {
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t);
    unsafe fn drop_with_sender(&mut self, recv: bindings::NDIlib_send_instance_t);
    fn assert_unwritten(&self);
}
