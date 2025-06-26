use std::ffi::{CStr, CString};

use crate::bindings;
pub use crate::bindings::NDIlib_metadata_frame_t as NDIRawMetadataFrame;

use super::{NDIFrame, NDIFrameExt, RawFrame, RawFrameInner, drop_guard::FrameDataDropGuard};

impl RawFrameInner for NDIRawMetadataFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_metadata(recv, self) }
    }

    #[inline]
    unsafe fn drop_with_sender(&mut self, sender: bindings::NDIlib_send_instance_t) {
        unsafe { bindings::NDIlib_send_free_metadata(sender, self) }
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "[Fatal FFI Error] NDIRawMetadataFrame data is not null, but should be."
        );
    }
}

impl RawFrame for NDIRawMetadataFrame {}

pub type MetadataFrame = NDIFrame<NDIRawMetadataFrame>;

impl NDIFrameExt<NDIRawMetadataFrame> for MetadataFrame {
    fn data_valid(&self) -> bool {
        !self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
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

    pub fn from_string(cstr: CString) -> Self {
        let raw = NDIRawMetadataFrame {
            timecode: bindings::NDIlib_send_timecode_synthesize,
            p_data: cstr.as_ptr() as *mut _,
            length: cstr.as_bytes_with_nul().len() as i32,
        };
        Self {
            raw,
            alloc: FrameDataDropGuard::CString(cstr),
            custom_state: (),
        }
    }

    pub fn to_str(&self) -> Option<&CStr> {
        if let FrameDataDropGuard::CString(cstr) = &self.alloc {
            return Some(cstr.as_c_str());
        }
        if self.raw.p_data.is_null() {
            None
        } else {
            let str = unsafe { CStr::from_ptr(self.raw.p_data as *const _) };
            if self.raw.length > 0 {
                assert_eq!(
                    str.to_bytes_with_nul().len(),
                    self.raw.length as usize,
                    "[Fatal FFI Error] NDIFrame::to_cstr: length mismatch"
                );
            }
            Some(str)
        }
    }
}

impl From<CString> for MetadataFrame {
    fn from(cstr: CString) -> Self {
        Self::from_string(cstr)
    }
}
