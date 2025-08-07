use std::{ffi::CString, fmt::Debug, sync::Arc};

use crate::{receiver::RawReceiver, sender::RawSender};

/// Holds the frame allocation
#[derive(PartialEq, Eq)]
pub(crate) enum FrameDataDropGuard {
    NullPtr,
    Receiver(Option<Arc<RawReceiver>>),
    Sender(Option<Arc<RawSender>>),
    Box(Box<[u8]>),
    CString(CString),
}

unsafe impl Send for FrameDataDropGuard {}
unsafe impl Sync for FrameDataDropGuard {}

impl Debug for FrameDataDropGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullPtr => write!(f, "NullPtr"),
            Self::Receiver(recv) => f.debug_tuple("Receiver").field(recv).finish(),
            Self::Sender(sender) => f.debug_tuple("Sender").field(sender).finish(),
            Self::Box(data) => write!(f, "Box ({} bytes)", data.len()),
            Self::CString(cstr) => write!(f, "CString ({})", cstr.to_string_lossy()),
        }
    }
}

// To drop the recv/sender frames we need additional information. Therefore they are dropped explicitly by calling `drop_buffer`.
// This drop impl just checks if this was done correctly.
impl Drop for FrameDataDropGuard {
    fn drop(&mut self) {
        match self {
            Self::Receiver(Some(recv)) => {
                panic!(
                    "Attempted to drop FrameDataDropGuard::Receiver, the frame was not freed correctly: {:?}",
                    recv
                );
            }

            Self::Sender(Some(sender)) => {
                panic!(
                    "Attempted to drop FrameDataDropGuard::Sender, the frame was not freed correctly: {:?}",
                    sender
                );
            }

            _ => {}
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

    /// Check if the frame is allocated (=it is not a NullPtr)
    #[inline]
    pub fn is_allocated(&self) -> bool {
        !matches!(self, FrameDataDropGuard::NullPtr)
    }

    #[inline]
    pub fn is_from_sdk(&self) -> bool {
        matches!(
            self,
            FrameDataDropGuard::Receiver(_) | FrameDataDropGuard::Sender(_)
        )
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

    /// Generate Error message
    /// panics if the frame is readable
    pub fn reason_for_not_ffi_readable(&self) -> &'static str {
        match self {
            FrameDataDropGuard::NullPtr => "Frame is not allocated",
            _ => {
                panic!("Frame is ffi readable, but trying to get reason why not")
            }
        }
    }

    pub fn from_sender(&mut self, sender: Arc<RawSender>) {
        *self = FrameDataDropGuard::Sender(Some(sender));
    }

    pub fn from_receiver(&mut self, recv: Arc<RawReceiver>) {
        *self = FrameDataDropGuard::Receiver(Some(recv));
    }

    /// Drop the buffer, freeing the memory if necessary
    pub unsafe fn drop_buffer(&mut self, raw: &mut impl RawBufferManagement) {
        if let FrameDataDropGuard::Receiver(recv) = self
            && let Some(recv) = recv.take()
        {
            unsafe { raw.drop_with_recv(&recv) }
        } else if let FrameDataDropGuard::Sender(sender) = self
            && let Some(sender) = sender.take()
        {
            unsafe { raw.drop_with_sender(&sender) };
        }

        // self and with it all owned data is dropped
        *self = FrameDataDropGuard::NullPtr;
    }
}

pub(crate) trait RawBufferManagement {
    unsafe fn drop_with_recv(&mut self, recv: &Arc<RawReceiver>);
    unsafe fn drop_with_sender(&mut self, sender: &Arc<RawSender>);
    fn assert_unwritten(&self);
}
