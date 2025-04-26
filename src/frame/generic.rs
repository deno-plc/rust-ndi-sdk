use super::{RawFrame, drop_guard::FrameDataDropGuard};

unsafe impl<Raw: RawFrame, C> Send for NDIFrame<Raw, C> where C: Send {}

pub struct NDIFrame<Raw: RawFrame, C = ()> {
    pub(crate) raw: Raw,
    pub(crate) alloc: FrameDataDropGuard,
    #[allow(unused)]
    pub(crate) custom_state: C,
}

impl<Raw: RawFrame> NDIFrame<Raw> {
    #[inline]
    pub fn is_ffi_writable(&self) -> bool {
        self.alloc.is_ffi_writable()
    }

    pub(crate) fn assert_unwritten(&self) {
        self.raw.assert_unwritten();
        assert!(
            self.alloc.is_ffi_writable(),
            "NDIFrame is not writable, but should be. This is a bug, most likely due to an FFI contract violation."
        );
    }

    pub(crate) unsafe fn drop_buffer_backend(&mut self) {
        unsafe { FrameDataDropGuard::drop_with_recv(&mut self.alloc, &mut self.raw) };
    }
}

impl<Raw: RawFrame, C> Drop for NDIFrame<Raw, C> {
    fn drop(&mut self) {
        match &self.alloc {
            FrameDataDropGuard::Receiver(recv) => {
                unsafe { self.raw.drop_with_recv(*recv) };
            }
            FrameDataDropGuard::Box(_) | FrameDataDropGuard::NullPtr => {}
        }
    }
}

pub(crate) trait AsFFIWritable<T: RawFrame> {
    fn to_ffi_frame_ptr(&mut self) -> *mut T;
}

impl<Raw: RawFrame> AsFFIWritable<Raw> for NDIFrame<Raw> {
    fn to_ffi_frame_ptr(&mut self) -> *mut Raw {
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
    fn to_ffi_frame_ptr(&mut self) -> *mut Raw {
        if let Some(frame) = self {
            frame.to_ffi_frame_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

impl<Raw: RawFrame> AsFFIWritable<Raw> for Option<&mut NDIFrame<Raw>> {
    fn to_ffi_frame_ptr(&mut self) -> *mut Raw {
        if let Some(frame) = self {
            frame.to_ffi_frame_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}
