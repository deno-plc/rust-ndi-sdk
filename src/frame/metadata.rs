use crate::bindings;
pub use crate::bindings::NDIlib_metadata_frame_t as NDIRawMetadataFrame;

use super::{NDIFrame, NDIFrameExt, RawFrame, RawFrameInner, drop_guard::FrameDataDropGuard};

impl RawFrameInner for NDIRawMetadataFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_metadata(recv, self) }
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "NDIRawMetadataFrame data is not null, but should be. This is a bug, most likely due to an FFI contract violation."
        );
    }
}

impl RawFrame for NDIRawMetadataFrame {}

pub type MetadataFrame = NDIFrame<NDIRawMetadataFrame>;

impl NDIFrameExt<NDIRawMetadataFrame> for MetadataFrame {
    fn data_valid(&self) -> bool {
        self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
    }

    fn clear(&mut self) {
        unsafe { self.drop_buffer_backend() };
        self.raw.p_data = std::ptr::null_mut();
    }
}

impl MetadataFrame {
    pub fn new() -> Self {
        let raw = NDIRawMetadataFrame {
            timecode: bindings::NDIlib_send_timecode_synthesize,
            p_data: std::ptr::null_mut(),
            length: 0,
        };
        Self {
            raw,
            alloc: FrameDataDropGuard::NullPtr,
            custom_state: (),
        }
    }
}
