use std::{ffi::CString, fmt::Debug};

use crate::bindings;

use super::RawFrame;

/// Holds the frame allocation
#[derive(PartialEq, Eq)]
pub enum FrameDataDropGuard {
    NullPtr,
    Receiver(bindings::NDIlib_recv_instance_t),
    Sender(bindings::NDIlib_send_instance_t),
    Box(Box<[u8]>),
    CString(CString),
}

impl Debug for FrameDataDropGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullPtr => write!(f, "NullPtr"),
            Self::Receiver(recv) => f.debug_tuple("Receiver").field(recv).finish(),
            Self::Sender(recv) => f.debug_tuple("Sender").field(recv).finish(),
            Self::Box(data) => write!(f, "Box ({} bytes)", data.len()),
            Self::CString(cstr) => write!(f, "CString ({})", cstr.to_string_lossy()),
        }
    }
}

impl Drop for FrameDataDropGuard {
    fn drop(&mut self) {
        match self {
            Self::Receiver(recv) => {
                if !recv.is_null() {
                    panic!(
                        "Attempted to drop FrameDataDropGuard::Receiver, the frame was not freed correctly: {:?}",
                        recv
                    );
                }
            }

            Self::Sender(sender) => {
                if !sender.is_null() {
                    panic!(
                        "Attempted to drop FrameDataDropGuard::Sender, the frame was not freed correctly: {:?}",
                        sender
                    );
                }
            }

            Self::Box(_) | Self::CString(_) => {}
            Self::NullPtr => {}
        }
    }
}

impl FrameDataDropGuard {
    /// allocate a new frame hold in a Box
    pub fn new_boxed(size: usize) -> (FrameDataDropGuard, *mut u8) {
        let mut buf = vec![0u8; size].into_boxed_slice();
        let ptr = buf.as_mut_ptr();
        (FrameDataDropGuard::Box(buf), ptr)
    }

    /// Check if the frame is writable by FFI (=it is a NullPtr)
    #[inline]
    pub fn is_ffi_writable(&self) -> bool {
        matches!(self, FrameDataDropGuard::NullPtr)
    }

    /// Check if the frame is readable by FFI (=it is not a NullPtr)
    #[inline]
    pub fn is_ffi_readable(&self) -> bool {
        !matches!(self, FrameDataDropGuard::NullPtr)
    }

    /// Checks if it is safe to write into the frame data by user code
    #[inline]
    pub fn is_mut(&self) -> bool {
        matches!(
            self,
            FrameDataDropGuard::Box(_) | FrameDataDropGuard::CString(_)
        )
    }

    /// Generate Error message
    /// panics if the frame is writable
    pub fn reason_for_not_ffi_writable(&self) -> &'static str {
        match self {
            FrameDataDropGuard::NullPtr => {
                panic!("Frame is ffi writable, but trying to get reason why not")
            }
            FrameDataDropGuard::Receiver(_) => "Already written by receiver",
            FrameDataDropGuard::Sender(_) => "Already written by sender",
            FrameDataDropGuard::Box(_) => "Data is Boxed, intended to be sent",
            FrameDataDropGuard::CString(_) => "Data is CString, intended to be sent",
        }
    }

    /// Drop the buffer, freeing the memory if necessary
    pub unsafe fn drop_buffer(&mut self, raw: &mut impl RawFrame) {
        if let FrameDataDropGuard::Receiver(recv) = self {
            unsafe { raw.drop_with_recv(*recv) }
            *recv = std::ptr::null_mut();
        } else if let FrameDataDropGuard::Sender(sender) = self {
            unsafe { raw.drop_with_sender(*sender) }
            *sender = std::ptr::null_mut();
        }

        // self and with it the Box is dropped here
        *self = FrameDataDropGuard::NullPtr;
    }
}
