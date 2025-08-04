use std::fmt::{Debug, Display};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    bindings::{self, NDIlib_FourCC_audio_type_e, NDIlib_FourCC_video_type_e},
    enums::NDIFieldedFrameMode,
    structs::{buffer_info::BufferInfo, resolution::Resolution, subsampling::Subsampling},
};

#[cfg(test)]
use strum::{EnumIter, IntoEnumIterator};

///! FourCC (Four Character Code) is a sequence of four bytes used to uniquely identify data formats.

/// Possible FourCC values for video frames.
#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(test, derive(EnumIter))]
pub enum FourCCVideo {
    UYVY = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_UYVY,
    UYVA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_UYVA,
    P216 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_P216,
    PA16 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_PA16,
    YV12 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_YV12,
    I420 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_I420,
    NV12 = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_NV12,
    BGRA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_BGRA,
    BGRX = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_BGRX,
    RGBA = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_RGBA,
    RGBX = bindings::NDIlib_FourCC_video_type_e_NDIlib_FourCC_video_type_RGBX,
}

impl FourCCVideo {
    pub(crate) fn to_ffi(self) -> NDIlib_FourCC_video_type_e {
        self.into()
    }

    pub(crate) fn from_ffi(value: NDIlib_FourCC_video_type_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }

    pub fn buffer_size(self, resolution: Resolution) -> Option<usize> {
        use FourCCVideo::*;
        match self {
            UYVY => Some(resolution.pixels() * 2),
            BGRA | BGRX | RGBA | RGBX => Some(resolution.pixels() * 4),
            _ => None,
        }
    }

    /// Returns information about the memory layout of the frame data
    pub fn buffer_info(
        self,
        resolution: Resolution,
        field_mode: NDIFieldedFrameMode,
    ) -> Result<BufferInfo, BufferInfoError> {
        use FourCCVideo::*;
        let mut pixel_stride = 0;
        let mut subsampling = Subsampling::default();

        match self {
            UYVY => {
                pixel_stride = 2;
                subsampling = Subsampling::new(4, 2, 2);
                Ok(())
            }
            BGRA | BGRX | RGBA | RGBX => {
                pixel_stride = 4;
                Ok(())
            }
            cc => Err(BufferInfoError::UnsupportedFourCC(cc)),
        }?;

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
    UnsupportedFourCC(FourCCVideo),
    UnspecifiedFourCC,
}

#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(test, derive(EnumIter))]
pub enum FourCCAudio {
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

/// Make sure that the video and audio FourCCs do not overlap, otherwise FourCC::from_ffi
/// will not be able to distinguish between them.
#[test]
fn four_cc_no_overlap() {
    use std::collections::HashSet;

    let video = FourCCVideo::iter()
        .map(|variant| variant.to_ffi())
        .collect::<HashSet<_>>();
    let audio = FourCCAudio::iter()
        .map(|variant| variant.to_ffi())
        .collect::<HashSet<_>>();

    assert_eq!(video.intersection(&audio).count(), 0);
}

#[non_exhaustive]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum FourCC {
    Video(FourCCVideo),
    Audio(FourCCAudio),
    Unknown(i32),
}

impl FourCC {
    pub fn to_ffi(self) -> i32 {
        match self {
            FourCC::Video(cc) => cc.to_ffi(),
            FourCC::Audio(cc) => cc.to_ffi(),
            FourCC::Unknown(cc) => cc,
        }
    }

    pub fn from_ffi(value: NDIlib_FourCC_audio_type_e) -> Self {
        if let Some(cc) = FourCCVideo::from_ffi(value) {
            FourCC::Video(cc)
        } else if let Some(cc) = FourCCAudio::from_ffi(value) {
            FourCC::Audio(cc)
        } else {
            FourCC::Unknown(value)
        }
    }

    pub fn to_string(self) -> String {
        let bytes: [u8; 4] = self.to_ffi().to_le_bytes();

        String::from_utf8_lossy(&bytes).to_string()
    }

    fn type_name(self) -> &'static str {
        match self {
            FourCC::Video(_) => "Video",
            FourCC::Audio(_) => "Audio",
            FourCC::Unknown(_) => "Unknown",
        }
    }
}

impl Display for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: [u8; 4] = self.to_ffi().to_le_bytes();

        let ascii = String::from_utf8_lossy(&bytes);

        write!(f, "{}", ascii)
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: [u8; 4] = self.to_ffi().to_le_bytes();

        let ascii = String::from_utf8_lossy(&bytes);

        write!(f, "FourCC({},{})", ascii, self.type_name())
    }
}

impl From<FourCCVideo> for FourCC {
    fn from(value: FourCCVideo) -> Self {
        FourCC::Video(value)
    }
}

impl From<FourCCAudio> for FourCC {
    fn from(value: FourCCAudio) -> Self {
        FourCC::Audio(value)
    }
}
