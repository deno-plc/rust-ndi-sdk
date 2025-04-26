use std::fmt::Debug;
use std::pin::Pin;

use crate::bindings;

use super::RawFrame;

#[derive(PartialEq, Eq)]
pub enum FrameDataDropGuard {
    NullPtr,
    Receiver(bindings::NDIlib_recv_instance_t),
    Box(Pin<Box<[u8]>>),
}

impl Debug for FrameDataDropGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullPtr => write!(f, "NullPtr"),
            Self::Receiver(recv) => f.debug_tuple("Receiver").field(recv).finish(),
            Self::Box(data) => write!(f, "Box ({} bytes)", data.len()),
        }
    }
}

impl FrameDataDropGuard {
    pub fn new_boxed(size: usize) -> (FrameDataDropGuard, *mut u8) {
        let mut buf = Pin::new(vec![0u8; size].into_boxed_slice());
        let ptr = buf.as_mut_ptr();
        (FrameDataDropGuard::Box(buf), ptr)
    }

    #[inline]
    pub fn is_ffi_writable(&self) -> bool {
        matches!(self, FrameDataDropGuard::NullPtr)
    }

    /// Generate Error message
    /// panics if the frame is writable
    pub fn reason_for_not_ffi_writable(&self) -> &'static str {
        match self {
            FrameDataDropGuard::NullPtr => {
                panic!("Frame is ffi writable, but trying to get reason why not")
            }
            FrameDataDropGuard::Receiver(_) => "Already written by receiver",
            FrameDataDropGuard::Box(_) => "Data is Boxed, intended to be sent",
        }
    }

    pub unsafe fn drop_with_recv(&mut self, raw: &mut impl RawFrame) {
        if let FrameDataDropGuard::Receiver(recv) = self {
            unsafe { raw.drop_with_recv(*recv) }
        }

        // self and with it the Box is dropped here
        *self = FrameDataDropGuard::NullPtr;
    }
}
