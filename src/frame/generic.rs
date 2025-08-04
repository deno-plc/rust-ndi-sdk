use super::{RawFrame, drop_guard::FrameDataDropGuard};

unsafe impl<Raw: RawFrame, C> Send for NDIFrame<Raw, C> where C: Send {}

pub struct NDIFrame<Raw: RawFrame, C = ()> {
    pub(crate) raw: Raw,
    pub(crate) alloc: FrameDataDropGuard,
    #[allow(unused)]
    pub(crate) custom_state: C,
}

impl<Raw: RawFrame> NDIFrame<Raw> {
    /// Checks if this frame can be written/received to by the SDK
    #[inline]
    pub fn is_ffi_writable(&self) -> bool {
        self.alloc.is_ffi_writable()
    }

    /// Checks if this frame can be read/sent by the SDK
    #[inline]
    pub fn is_ffi_readable(&self) -> bool {
        self.alloc.is_ffi_readable()
    }

    /// Checks if this frame is allocated/can be read by the user code
    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.alloc.is_allocated()
    }

    pub(crate) fn assert_unwritten(&self) {
        self.raw.assert_unwritten();
        assert!(
            self.alloc.is_ffi_writable(),
            "NDIFrame is not writable, but should be. This is a bug, most likely due to an FFI contract violation."
        );
    }
}

impl<Raw: RawFrame, C> Drop for NDIFrame<Raw, C> {
    fn drop(&mut self) {
        unsafe { self.alloc.drop_buffer(&mut self.raw) };
    }
}

pub(crate) trait AsFFIWritable<T: RawFrame> {
    fn to_ffi_recv_frame_ptr(&mut self) -> *mut T;
}

impl<Raw: RawFrame> AsFFIWritable<Raw> for NDIFrame<Raw> {
    fn to_ffi_recv_frame_ptr(&mut self) -> *mut Raw {
        if self.is_ffi_writable() {
            &mut self.raw
        } else {
            #[cfg(any(debug_assertions, feature = "strict_assertions"))]
            {
                panic!(
                    "NDIFrame is not writable, but passed to recv. ({})",
                    self.alloc.reason_for_not_ffi_writable()
                );
            }
            #[cfg(not(any(debug_assertions, feature = "strict_assertions")))]
            {
                eprintln!(
                    "NDIFrame is not writable, but passed to recv. Ignoring. ({})",
                    self.alloc.reason_for_not_ffi_writable()
                );
                std::ptr::null_mut()
            }
        }
    }
}

impl<Raw: RawFrame> AsFFIWritable<Raw> for Option<NDIFrame<Raw>> {
    fn to_ffi_recv_frame_ptr(&mut self) -> *mut Raw {
        if let Some(frame) = self {
            frame.to_ffi_recv_frame_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

impl<Raw: RawFrame> AsFFIWritable<Raw> for Option<&mut NDIFrame<Raw>> {
    fn to_ffi_recv_frame_ptr(&mut self) -> *mut Raw {
        if let Some(frame) = self {
            frame.to_ffi_recv_frame_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

pub(crate) trait AsFFIReadable<T: RawFrame> {
    fn to_ffi_send_frame_ptr(&self) -> Result<*const T, FFIReadablePtrError>;
}

impl<Raw: RawFrame> AsFFIReadable<Raw> for NDIFrame<Raw> {
    fn to_ffi_send_frame_ptr(&self) -> Result<*const Raw, FFIReadablePtrError> {
        if self.is_ffi_readable() {
            let ptr: *const Raw = &self.raw;
            assert!(!ptr.is_null());
            Ok(ptr)
        } else {
            Err(FFIReadablePtrError::NotReadable(
                self.alloc.reason_for_not_ffi_readable(),
            ))
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FFIReadablePtrError {
    NotReadable(&'static str),
}
