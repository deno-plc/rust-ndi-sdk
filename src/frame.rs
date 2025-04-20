use std::pin::Pin;
use std::{ffi::CStr, fmt::Debug};

use num::{Rational32, ToPrimitive};

use crate::structs::Resolution;
use crate::timecode::NDITime;
use crate::{
    bindings,
    enums::NDIFieldedFrameMode,
    four_cc::{FourCCAudio, FourCCVideo},
};

pub use crate::bindings::{
    NDIlib_audio_frame_v3_t as NDIRawAudioFrame, NDIlib_metadata_frame_t as NDIRawMetadataFrame,
    NDIlib_video_frame_v2_t as NDIRawVideoFrame,
};

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
            Self::Receiver(arg0) => f.debug_tuple("Receiver").field(arg0).finish(),
            Self::Box(data) => write!(f, "Box ({} bytes)", data.len()),
        }
    }
}

impl FrameDataDropGuard {
    #[inline]
    fn is_ffi_writable(&self) -> bool {
        matches!(self, FrameDataDropGuard::NullPtr)
    }

    /// Generate Error message
    fn reason_for_not_ffi_writable(&self) -> &'static str {
        match self {
            FrameDataDropGuard::NullPtr => {
                panic!("Frame is ffi writable, but trying to get reason why not")
            }
            FrameDataDropGuard::Receiver(_) => "Already written by receiver",
            FrameDataDropGuard::Box(_) => "Data is Boxed, intended to be sent",
        }
    }
}

#[allow(private_bounds)]
pub trait RawFrame: RawFrameInner {}

pub(crate) trait RawFrameInner {
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t);
    fn assert_unwritten(&self);
}

impl RawFrameInner for NDIRawVideoFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_video_v2(recv, self) }
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "NDIRawVideoFrame data is not null, but should be. This is a bug, most likely due to an FFI contract violation."
        );
        assert!(
            self.p_metadata.is_null(),
            "NDIRawVideoFrame metadata is not null, but should be. This is a bug, most likely due to an FFI contract violation."
        );
    }
}

impl RawFrame for NDIRawVideoFrame {}

impl RawFrameInner for NDIRawAudioFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_audio_v3(recv, self) }
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "NDIRawAudioFrame data is not null, but should be. This is a bug, most likely due to an FFI contract violation."
        );
        assert!(
            self.p_metadata.is_null(),
            "NDIRawAudioFrame metadata is not null, but should be. This is a bug, most likely due to an FFI contract violation."
        );
    }
}

impl RawFrame for NDIRawAudioFrame {}

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

pub(crate) trait AsFFIWritable<T: RawFrame> {
    fn to_ffi_frame_ptr(&mut self) -> *mut T;
}

pub struct NDIFrame<Raw: RawFrame, C = ()> {
    pub(crate) raw: Raw,
    pub(crate) alloc: FrameDataDropGuard,
    #[allow(unused)]
    pub(crate) custom_state: C,
}

unsafe impl<Raw: RawFrame, C> Send for NDIFrame<Raw, C> where C: Send {}

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

    unsafe fn drop_buffer_backend(&mut self) {
        if let FrameDataDropGuard::Receiver(recv) = self.alloc {
            unsafe { self.raw.drop_with_recv(recv) }
        }

        // this drops Box
        self.alloc = FrameDataDropGuard::NullPtr;
    }
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

pub trait NDIFrameExt<T: RawFrame> {
    fn data_valid(&self) -> bool;

    /// Frees the buffers
    /// Note: Other properties like resolution, frame rate and FourCC might or might not be retained
    fn clear(&mut self);
}

pub type VideoFrame = NDIFrame<NDIRawVideoFrame>;

impl NDIFrameExt<NDIRawVideoFrame> for VideoFrame {
    fn data_valid(&self) -> bool {
        self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
    }

    fn clear(&mut self) {
        unsafe { self.drop_buffer_backend() };
        self.raw.p_data = std::ptr::null_mut();
        self.raw.p_metadata = std::ptr::null_mut();
    }
}

impl VideoFrame {
    pub fn new() -> Self {
        let raw = NDIRawVideoFrame {
            xres: 0,
            yres: 0,
            FourCC: FourCCVideo::UYVY.to_ffi(),
            frame_rate_N: 30_000,
            frame_rate_D: 1001,
            picture_aspect_ratio: 0.0,
            frame_format_type: NDIFieldedFrameMode::Progressive.to_ffi(),
            timecode: bindings::NDIlib_send_timecode_synthesize,
            p_metadata: std::ptr::null_mut(),
            p_data: std::ptr::null_mut(),
            __bindgen_anon_1: bindings::NDIlib_video_frame_v2_t__bindgen_ty_1 {
                line_stride_in_bytes: 0,
            },
            timestamp: 0,
        };
        Self {
            raw,
            alloc: FrameDataDropGuard::NullPtr,
            custom_state: (),
        }
    }

    pub fn resolution(&self) -> Resolution {
        Resolution::new(
            self.raw
                .xres
                .try_into()
                .expect("Unexpected negative x-resolution, failed to cast to usize"),
            self.raw
                .yres
                .try_into()
                .expect("Unexpected negative y-resolution, failed to cast to usize"),
        )
    }

    pub fn four_cc(&self) -> Option<FourCCVideo> {
        FourCCVideo::from_ffi(self.raw.FourCC)
    }

    pub fn frame_rate(&self) -> Rational32 {
        Rational32::new_raw(self.raw.frame_rate_N, self.raw.frame_rate_D)
    }

    pub fn metadata(&self) -> Option<&CStr> {
        if self.raw.p_metadata.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.raw.p_metadata) })
        }
    }

    pub fn frame_format(&self) -> Option<NDIFieldedFrameMode> {
        NDIFieldedFrameMode::from_ffi(self.raw.frame_format_type)
    }

    pub fn send_time(&self) -> NDITime {
        NDITime::from_ffi(self.raw.timecode)
    }

    pub fn recv_time(&self) -> NDITime {
        NDITime::from_ffi(self.raw.timestamp)
    }

    fn stride(&self) -> i32 {
        unsafe { self.raw.__bindgen_anon_1.line_stride_in_bytes }
    }

    pub fn video_data(&self) -> Option<&[u8]> {
        if self.raw.p_data.is_null() {
            None
        } else if let Some(buffer_size) = self
            .four_cc()
            .and_then(|cc| cc.buffer_size(self.resolution()))
        {
            Some(unsafe { std::slice::from_raw_parts(self.raw.p_data, buffer_size) })
        } else {
            None
        }
    }
}

impl Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VideoFrame {{ ")?;

        write!(f, "resolution: {}x{}, ", self.raw.xres, self.raw.yres)?;

        write!(
            f,
            "frame rate: {:.2}fps, ",
            self.frame_rate().to_f64().unwrap_or(-1.)
        )?;

        if let Some(cc) = self.four_cc() {
            write!(f, "FourCC: {:?}, ", cc)?;
        } else {
            write!(f, "FourCC: {:#x}, ", self.raw.FourCC)?;
        }

        write!(f, "format: {:?}, ", self.frame_format().unwrap())?;

        write!(f, "stride: {}, ", self.stride())?;

        write!(f, "metadata: {:?}, ", self.metadata())?;

        write!(
            f,
            "timing: send={:?} recv={:?}, ",
            self.send_time(),
            self.recv_time()
        )?;

        write!(f, "alloc: {:?} }}", self.alloc)
    }
}

pub type AudioFrame = NDIFrame<NDIRawAudioFrame>;

impl NDIFrameExt<NDIRawAudioFrame> for AudioFrame {
    fn data_valid(&self) -> bool {
        self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
    }

    fn clear(&mut self) {
        unsafe { self.drop_buffer_backend() };
        self.raw.p_data = std::ptr::null_mut();
        self.raw.p_metadata = std::ptr::null_mut();
    }
}

impl AudioFrame {
    pub fn new() -> Self {
        let raw = NDIRawAudioFrame {
            sample_rate: 48_000,
            no_channels: 2,
            no_samples: 0,
            timecode: bindings::NDIlib_send_timecode_synthesize,
            timestamp: 0,
            FourCC: FourCCAudio::FLTP.to_ffi(),
            p_data: std::ptr::null_mut(),
            p_metadata: std::ptr::null_mut(),
            __bindgen_anon_1: bindings::NDIlib_audio_frame_v3_t__bindgen_ty_1 {
                channel_stride_in_bytes: 0,
            },
        };
        Self {
            raw,
            alloc: FrameDataDropGuard::NullPtr,
            custom_state: (),
        }
    }

    pub fn four_cc(&self) -> Option<FourCCAudio> {
        FourCCAudio::from_ffi(self.raw.FourCC)
    }
}

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
