use num::ToPrimitive;
use std::ffi::CStr;
use std::fmt::Debug;

use num::Rational32;

pub use crate::bindings::NDIlib_video_frame_v2_t as NDIRawVideoFrame;
use crate::{
    bindings,
    enums::NDIFieldedFrameMode,
    four_cc::FourCCVideo,
    structs::{buffer_info::BufferInfo, resolution::Resolution},
    timecode::NDITime,
};

use super::{NDIFrame, NDIFrameExt, RawFrame, RawFrameInner, drop_guard::FrameDataDropGuard};

impl RawFrameInner for NDIRawVideoFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_video_v2(recv, self) }
    }

    #[inline]
    unsafe fn drop_with_sender(&mut self, _sender: bindings::NDIlib_send_instance_t) {
        panic!(
            "NDIRawVideoFrame cannot be dropped with a sender as it cannot be received by the sender."
        )
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "[Fatal FFI Error] NDIRawVideoFrame data is not null, but should be."
        );
        assert!(
            self.p_metadata.is_null(),
            "[Fatal FFI Error] NDIRawVideoFrame metadata is not null, but should be."
        );
    }
}

impl RawFrame for NDIRawVideoFrame {}

pub type VideoFrame = NDIFrame<NDIRawVideoFrame>;

impl NDIFrameExt<NDIRawVideoFrame> for VideoFrame {
    fn data_valid(&self) -> bool {
        !self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
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

    pub unsafe fn alloc_raw_frame_buffer(
        &mut self,
        size: usize,
        stride: i32,
        resolution: Resolution,
    ) {
        let (alloc, ptr) = FrameDataDropGuard::new_boxed(size);
        self.alloc = alloc;
        self.raw.p_data = ptr;
        unsafe {
            self.set_lib_stride(stride);
            self.set_resolution(resolution);
        }
    }

    pub fn try_alloc(
        &mut self,
        resolution: Resolution,
        four_cc: FourCCVideo,
    ) -> Result<(), VideoFrameAllocationError> {
        if let Some(info) = four_cc.buffer_info(resolution, self.frame_format()) {
            let (alloc, ptr) = FrameDataDropGuard::new_boxed(info.size);
            unsafe { self.drop_buffer_backend() };
            self.alloc = alloc;
            self.raw.p_data = ptr;
            self.raw.FourCC = four_cc.to_ffi();
            unsafe {
                self.set_lib_stride(info.line_stride as i32);
                self.set_resolution(resolution);
            }
            Ok(())
        } else {
            Err(VideoFrameAllocationError::UnsupportedFourCC(four_cc))
        }
    }

    pub fn alloc(&mut self, resolution: Resolution, four_cc: FourCCVideo) {
        self.try_alloc(resolution, four_cc).unwrap();
    }

    pub fn resolution(&self) -> Resolution {
        Resolution::from_i32(self.raw.xres, self.raw.yres)
    }
    pub unsafe fn set_resolution(&mut self, resolution: Resolution) {
        (self.raw.xres, self.raw.yres) = resolution.to_i32();
    }

    pub fn four_cc(&self) -> Option<FourCCVideo> {
        FourCCVideo::from_ffi(self.raw.FourCC)
    }
    pub unsafe fn set_four_cc(&mut self, four_cc: FourCCVideo) {
        self.raw.FourCC = four_cc.to_ffi();
    }

    pub fn frame_rate(&self) -> Rational32 {
        Rational32::new_raw(self.raw.frame_rate_N, self.raw.frame_rate_D)
    }
    pub fn set_frame_rate(&mut self, frame_rate: Rational32) {
        self.raw.frame_rate_N = *frame_rate.numer();
        self.raw.frame_rate_D = *frame_rate.denom();
    }

    pub fn metadata(&self) -> Option<&CStr> {
        if self.raw.p_metadata.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.raw.p_metadata) })
        }
    }
    // TODO: pub fn set_metadata

    pub fn frame_format(&self) -> NDIFieldedFrameMode {
        NDIFieldedFrameMode::from_ffi(self.raw.frame_format_type)
            .expect("[Fatal FFI Error] Invalid frame format type")
    }
    pub unsafe fn set_frame_format(&mut self, frame_format: NDIFieldedFrameMode) {
        self.raw.frame_format_type = frame_format.to_ffi();
    }

    pub fn send_time(&self) -> NDITime {
        NDITime::from_ffi(self.raw.timecode)
    }
    pub fn set_send_time(&mut self, time: NDITime) {
        self.raw.timecode = time.to_ffi();
    }

    pub fn recv_time(&self) -> NDITime {
        NDITime::from_ffi(self.raw.timestamp)
    }
    pub fn set_recv_time(&mut self, time: NDITime) {
        self.raw.timestamp = time.to_ffi();
    }

    fn lib_stride(&self) -> i32 {
        unsafe { self.raw.__bindgen_anon_1.line_stride_in_bytes }
    }
    unsafe fn set_lib_stride(&mut self, stride: i32) {
        self.raw.__bindgen_anon_1.line_stride_in_bytes = stride;
    }

    pub fn video_data(&self) -> Option<(&[u8], BufferInfo)> {
        if self.raw.p_data.is_null() {
            None
        } else if let Some(info) = self
            .four_cc()
            .and_then(|cc| cc.buffer_info(self.resolution(), self.frame_format()))
        {
            assert_eq!(
                info.line_stride,
                self.lib_stride() as usize,
                "[Fatal FFI Error] Stride mismatch"
            );
            Some((
                unsafe { std::slice::from_raw_parts(self.raw.p_data, info.size) },
                info,
            ))
        } else {
            None
        }
    }

    pub fn video_data_mut(&mut self) -> Option<(&mut [u8], BufferInfo)> {
        if self.raw.p_data.is_null() {
            None
        } else if !self.alloc.is_mut() {
            None
        } else if let Some(info) = self
            .four_cc()
            .and_then(|cc| cc.buffer_info(self.resolution(), self.frame_format()))
        {
            assert_eq!(
                info.line_stride,
                self.lib_stride() as usize,
                "[Fatal FFI Error] Stride mismatch"
            );
            Some((
                unsafe { std::slice::from_raw_parts_mut(self.raw.p_data, info.size) },
                info,
            ))
        } else {
            None
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFrameAllocationError {
    UnsupportedFourCC(FourCCVideo),
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

        write!(f, "format: {:?}, ", self.frame_format())?;

        write!(f, "stride: {}, ", self.lib_stride())?;

        write!(f, "metadata: {:?}, ", self.metadata())?;

        write!(
            f,
            "timing: send={:?} recv={:?}, ",
            self.send_time(),
            self.recv_time()
        )?;

        write!(f, "alloc: {:?} @ {:?} }}", self.raw.p_data, self.alloc)
    }
}
