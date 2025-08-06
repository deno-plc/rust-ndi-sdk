//! FourCC (Four Character Code) is a sequence of four bytes used to uniquely identify data formats.
//!
//! https://docs.ndi.video/all/developing-with-ndi/sdk/frame-types#video-frames-ndilib_video_frame_v2_t
//! https://en.wikipedia.org/wiki/FourCC

use std::fmt::{Debug, Display};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    bindings::{self, NDIlib_FourCC_audio_type_e, NDIlib_FourCC_video_type_e},
    enums::NDIFieldedFrameMode,
    structs::{buffer_info::BufferInfo, resolution::Resolution, subsampling::Subsampling},
};

/// Possible FourCC values for video frames.
#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum FourCCVideo {
    UYVY = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_UYVY,
    UYVA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_UYVA,
    P216 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_P216,
    PA16 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_PA16,
    YV12 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_YV12,
    I420 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_I420,
    NV12 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_NV12,
    /// Red, Green, Blue, Alpha (8bit)
    RGBA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_RGBA,
    /// RGBA with ignored alpha channel
    RGBX = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_RGBX,
    /// Blue, Green, Red, Alpha (8bit)
    BGRA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_BGRA,
    /// BGRA with ignored alpha channel
    BGRX = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_BGRX,
}

impl FourCCVideo {
    pub(crate) fn to_ffi(self) -> NDIlib_FourCC_video_type_e {
        self.into()
    }

    pub(crate) fn from_ffi(value: NDIlib_FourCC_video_type_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }

    /// Returns information about the memory layout of the frame data
    pub fn buffer_info(
        self,
        resolution: Resolution,
        field_mode: NDIFieldedFrameMode,
    ) -> Result<BufferInfo, BufferInfoError> {
        use FourCCVideo::*;
        let pixel_stride;
        let mut subsampling = Subsampling::none();

        match self {
            UYVY => {
                pixel_stride = 2;
                subsampling = Subsampling::new(4, 2, 2);
            }
            BGRA | BGRX | RGBA | RGBX => {
                pixel_stride = 4;
            }
            cc => return Err(BufferInfoError::UnsupportedFourCC(cc)),
        };

        let size = resolution.pixels() * pixel_stride;

        Ok(BufferInfo {
            size: if field_mode.is_single_field() {
                size / 2
            } else {
                size
            },
            line_stride: resolution.x * pixel_stride,
            resolution,
            field_mode,
            subsampling,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferInfoError {
    /// There is no Layout implementation for this FourCC yet
    UnsupportedFourCC(FourCCVideo),
    UnspecifiedFourCC,
}

/// Possible FourCC values for audio frames.
#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum FourCCAudio {
    /// Floating point
    #[default]
    FLTP = bindings::NDIlib_FourCC_audio_type_e_NDIlib_FourCC_audio_type_FLTP,
}

impl FourCCAudio {
    pub fn to_ffi(self) -> NDIlib_FourCC_audio_type_e {
        self.into()
    }

    pub fn from_ffi(value: NDIlib_FourCC_audio_type_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }
}

/// Represents a generic FourCC code.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FourCC {
    // use anything with 32bit, using signed reduces the number of casts
    code: i32,
}

impl FourCC {
    /// Creates a new FourCC from an i32 code.
    pub fn from_ffi(code: i32) -> Self {
        FourCC { code }
    }

    /// Converts the FourCC to its i32 representation.
    pub fn to_ffi(self) -> i32 {
        self.code
    }

    /// Attempts to convert the FourCC to a FourCCVideo.
    pub fn as_video(&self) -> Option<FourCCVideo> {
        FourCCVideo::from_ffi(self.code)
    }

    /// Attempts to convert the FourCC to a FourCCAudio.
    pub fn as_audio(&self) -> Option<FourCCAudio> {
        FourCCAudio::from_ffi(self.code)
    }

    /// formats the FourCC as a string.
    pub fn to_string(&self) -> String {
        let bytes: [u8; 4] = self.code.to_le_bytes();

        String::from_utf8_lossy(&bytes).to_string()
    }
}

impl Display for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: [u8; 4] = self.code.to_le_bytes();

        let ascii = String::from_utf8_lossy(&bytes);

        write!(f, "{}", ascii)
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: [u8; 4] = self.code.to_le_bytes();

        let ascii = String::from_utf8_lossy(&bytes);

        write!(f, "FourCC({})", ascii,)
    }
}

impl From<FourCCVideo> for FourCC {
    fn from(value: FourCCVideo) -> Self {
        FourCC {
            code: value.to_ffi(),
        }
    }
}

impl From<FourCCAudio> for FourCC {
    fn from(value: FourCCAudio) -> Self {
        FourCC {
            code: value.to_ffi(),
        }
    }
}

impl From<i32> for FourCC {
    fn from(code: i32) -> Self {
        FourCC { code }
    }
}

impl From<FourCC> for i32 {
    fn from(fourcc: FourCC) -> Self {
        fourcc.code
    }
}

impl TryFrom<FourCC> for FourCCVideo {
    type Error = ();

    fn try_from(value: FourCC) -> Result<Self, Self::Error> {
        FourCCVideo::from_ffi(value.code).ok_or(())
    }
}

impl TryFrom<FourCC> for FourCCAudio {
    type Error = ();

    fn try_from(value: FourCC) -> Result<Self, Self::Error> {
        FourCCAudio::from_ffi(value.code).ok_or(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fmt() {
        let fourcc = FourCC::from(FourCCVideo::RGBA);
        assert_eq!(fourcc.to_string(), "RGBA");
        assert_eq!(format!("{:?}", fourcc), "FourCC(RGBA)");
    }
}
