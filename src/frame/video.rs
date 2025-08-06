use num::ToPrimitive;
use std::ffi::CStr;
use std::fmt::Debug;

use num::Rational32;

pub(crate) use crate::bindings::NDIlib_video_frame_v2_t as NDIRawVideoFrame;
use crate::{
    bindings,
    enums::NDIFieldedFrameMode,
    four_cc::{BufferInfoError, FourCC, FourCCVideo},
    structs::{buffer_info::BufferInfo, resolution::Resolution},
    timecode::NDITime,
    util::VoidResult,
};

use super::{NDIFrame, RawBufferManagement, RawFrame, drop_guard::FrameDataDropGuard};

impl RawBufferManagement for NDIRawVideoFrame {
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

/// Represents a video frame in NDI.
/// C equivalent: `NDIlib_video_frame_v2_t`
pub type VideoFrame = NDIFrame<NDIRawVideoFrame>;

impl VideoFrame {
    /// Constructs a new video frame (without allocating a frame buffer)
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

    /// Generates a {BufferInfo} for the current resolution/FourCC/field mode
    pub fn buffer_info(&self) -> Result<BufferInfo, BufferInfoError> {
        if let Some(cc) = self.four_cc() {
            cc.buffer_info(self.resolution(), self.field_mode())
        } else {
            Err(BufferInfoError::UnspecifiedFourCC)
        }
    }

    /// Tries to allocate a frame buffer for the video frame.
    pub fn try_alloc(&mut self) -> Result<(), VideoFrameAllocationError> {
        if self.is_allocated() {
            Err(VideoFrameAllocationError::AlreadyAllocated)?;
        }

        let info = self
            .buffer_info()
            .map_err(|err| VideoFrameAllocationError::BufferInfoError(err))?;

        let (alloc, ptr) = FrameDataDropGuard::new_boxed(info.size);
        self.alloc = alloc;
        self.raw.p_data = ptr;
        self.raw.__bindgen_anon_1.line_stride_in_bytes = info.line_stride as i32;

        Ok(())
    }

    /// Allocates a frame buffer for the video frame. Panics if there is an error.
    pub fn alloc(&mut self) {
        self.try_alloc().unwrap();
    }

    /// Deallocates the frame buffer
    pub fn dealloc(&mut self) {
        let drops_metadata = self.alloc.is_from_sdk();
        unsafe { self.alloc.drop_buffer(&mut self.raw) };
        self.raw.p_data = std::ptr::null_mut();
        self.raw.__bindgen_anon_1.line_stride_in_bytes = -1;
        if drops_metadata {
            self.raw.p_metadata = std::ptr::null_mut();
        }
    }

    /// Read access to the frame data
    pub fn video_data(&self) -> Result<(&[u8], BufferInfo), VideoFrameAccessError> {
        if !self.is_allocated() {
            Err(VideoFrameAccessError::NotAllocated)?;
        }

        let info = self
            .buffer_info()
            .map_err(|err| VideoFrameAccessError::BufferInfoError(err))?;

        assert_eq!(
            info.line_stride,
            self.lib_stride() as usize,
            "[Fatal FFI Error] Stride mismatch"
        );

        assert!(
            !self.raw.p_data.is_null(),
            "[Invariant Error] data pointer does not match allocation"
        );
        Ok((
            unsafe { std::slice::from_raw_parts(self.raw.p_data, info.size) },
            info,
        ))
    }

    /// Mutable access to the frame data
    pub fn video_data_mut(&mut self) -> Result<(&mut [u8], BufferInfo), VideoFrameAccessError> {
        if !self.is_allocated() {
            Err(VideoFrameAccessError::NotAllocated)?;
        }

        if !self.alloc.is_mut() {
            Err(VideoFrameAccessError::Readonly)?;
        }

        let info = self
            .buffer_info()
            .map_err(|err| VideoFrameAccessError::BufferInfoError(err))?;

        assert_eq!(
            info.line_stride,
            self.lib_stride() as usize,
            "[Fatal FFI Error] Stride mismatch"
        );

        assert!(
            !self.raw.p_data.is_null(),
            "[Invariant Error] data pointer does not match allocation"
        );

        Ok((
            unsafe { std::slice::from_raw_parts_mut(self.raw.p_data, info.size) },
            info,
        ))
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFrameAllocationError {
    /// The frame is already allocated
    /// You have to deallocate it first
    AlreadyAllocated,
    /// An error occurred while trying to compute the buffer info
    BufferInfoError(BufferInfoError),
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFrameAccessError {
    /// It is impossible to get a reference to a frame buffer that does not exist
    NotAllocated,
    /// Only possible for {VideoFrame::video_data_mut} if the buffer is not
    /// intended to be modified (like a received frame)
    Readonly,
    /// An error occurred while trying to compute the buffer info
    BufferInfoError(BufferInfoError),
}

// Property accessors
impl VideoFrame {
    /// Gets the resolution of the frame.
    pub fn resolution(&self) -> Resolution {
        Resolution::from_i32(self.raw.xres, self.raw.yres)
    }

    /// Sets the resolution of the frame.
    /// This will fail if the frame is already allocated.
    pub fn set_resolution(&mut self, resolution: Resolution) -> VoidResult {
        if self.is_allocated() {
            Err(())
        } else {
            (self.raw.xres, self.raw.yres) = resolution.to_i32();

            // self.raw.picture_aspect_ratio = resolution.aspect_ratio() as f32;
            // Let NDI do this for us
            // This assignment is necessary in case a frame is received (which sets the aspect ratio),
            // then deallocated, and then reallocated with a different resolution.
            self.raw.picture_aspect_ratio = 0.0;
            Ok(())
        }
    }

    /// Gets the FourCC format of the frame
    pub fn four_cc(&self) -> Option<FourCCVideo> {
        FourCCVideo::from_ffi(self.raw.FourCC)
    }

    pub fn raw_four_cc(&self) -> FourCC {
        FourCC::from_ffi(self.raw.FourCC)
    }

    /// Sets the FourCC format of the frame.
    /// This will fail if the frame is already allocated.
    pub fn set_four_cc(&mut self, four_cc: FourCCVideo) -> VoidResult {
        if self.is_allocated() {
            Err(())
        } else {
            self.raw.FourCC = four_cc.to_ffi();
            Ok(())
        }
    }

    pub fn frame_rate(&self) -> Rational32 {
        Rational32::new_raw(self.raw.frame_rate_N, self.raw.frame_rate_D)
    }
    pub fn set_frame_rate(&mut self, frame_rate: Rational32) {
        self.raw.frame_rate_N = *frame_rate.numer();
        self.raw.frame_rate_D = *frame_rate.denom();
    }

    /// Access the metadata associated with the frame if any
    pub fn metadata(&self) -> Option<&CStr> {
        if self.raw.p_metadata.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self.raw.p_metadata) })
        }
    }
    // TODO: pub fn set_metadata

    /// gets the current field mode of the frame
    pub fn field_mode(&self) -> NDIFieldedFrameMode {
        NDIFieldedFrameMode::from_ffi(self.raw.frame_format_type)
            .expect("[Fatal FFI Error] Invalid frame format type")
    }
    /// sets the field mode of the frame.
    /// This will fail if the frame is already allocated.
    pub fn set_frame_format(&mut self, frame_format: NDIFieldedFrameMode) -> VoidResult {
        if self.is_allocated() {
            Err(())
        } else {
            self.raw.frame_format_type = frame_format.to_ffi();
            Ok(())
        }
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

    /// This is not relevant until you do stuff with the allocation
    fn lib_stride(&self) -> i32 {
        unsafe { self.raw.__bindgen_anon_1.line_stride_in_bytes }
    }
}

// Dangerous APIs
#[cfg(feature = "dangerous_apis")]
impl VideoFrame {
    /// Forcefully sets the resolution even if the frame is allocated.
    /// This should be used with extreme caution.
    /// If used incorrectly, it can lead to memory corruption by out-of-bounds access.
    pub unsafe fn force_set_resolution(&mut self, resolution: Resolution) {
        (self.raw.xres, self.raw.yres) = resolution.to_i32();
        self.raw.picture_aspect_ratio = resolution.aspect_ratio() as f32;
    }

    /// Forcefully sets the FourCC even if the frame is allocated.
    /// This should be used with extreme caution.
    /// If used incorrectly, it can lead to memory corruption by out-of-bounds access.
    pub unsafe fn force_set_four_cc(&mut self, four_cc: FourCCVideo) {
        self.raw.FourCC = four_cc.to_ffi();
    }

    /// Forcefully sets the FourCC even if the frame is allocated.
    /// This should be used with extreme caution.
    /// If used incorrectly, it can lead to memory corruption by out-of-bounds access.
    pub unsafe fn force_set_raw_four_cc(&mut self, four_cc: FourCC) {
        self.raw.FourCC = four_cc.to_ffi();
    }

    /// Forcefully sets the field mode even if the frame is allocated.
    /// This should be used with extreme caution.
    /// If used incorrectly, it can lead to memory corruption by out-of-bounds access.
    pub unsafe fn force_set_frame_format(&mut self, frame_format: NDIFieldedFrameMode) {
        self.raw.frame_format_type = frame_format.to_ffi();
    }

    /// This should be used with extreme caution.
    /// If used incorrectly, it can lead to memory corruption by out-of-bounds access.
    unsafe fn set_lib_stride(&mut self, stride: i32) {
        self.raw.__bindgen_anon_1.line_stride_in_bytes = stride;
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

        write!(f, "format: {:?}, ", self.field_mode())?;

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
