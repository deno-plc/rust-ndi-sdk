pub mod audio;
pub(crate) mod drop_guard;
pub mod generic;
pub mod metadata;
pub mod video;

use crate::frame::drop_guard::RawBufferManagement;

pub use generic::NDIFrame;

#[allow(private_bounds)]
pub trait RawFrame: RawBufferManagement + Send + Sync {}
