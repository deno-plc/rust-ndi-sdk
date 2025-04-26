use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    bindings::{
        self, NDIlib_frame_format_type_e, NDIlib_recv_bandwidth_e, NDIlib_recv_color_format_e,
    },
    four_cc::FourCCVideo,
};

#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum NDIColorFormat {
    #[default]
    Fastest = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_fastest,
    Best = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_best,

    BGRX_BGRA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_BGRX_BGRA,
    UYVY_BGRA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_UYVY_BGRA,
    RGBX_RGBA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_RGBX_RGBA,
    UYVY_RGBA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_UYVY_RGBA,
}

impl NDIColorFormat {
    pub fn to_ffi(self) -> NDIlib_recv_color_format_e {
        self.into()
    }

    pub fn from_ffi(value: NDIlib_recv_color_format_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }

    /// None indicates that the resulting FourCC type cannot statically be known
    pub const fn without_alpha_four_cc(self) -> Option<FourCCVideo> {
        match self {
            NDIColorFormat::BGRX_BGRA => Some(FourCCVideo::BGRX),
            NDIColorFormat::RGBX_RGBA => Some(FourCCVideo::RGBX),
            NDIColorFormat::UYVY_BGRA | NDIColorFormat::UYVY_RGBA => Some(FourCCVideo::UYVY),
            _ => None,
        }
    }

    /// None indicates that the resulting FourCC type cannot statically be known
    pub const fn with_alpha_four_cc(self) -> Option<FourCCVideo> {
        match self {
            NDIColorFormat::BGRX_BGRA | NDIColorFormat::UYVY_BGRA => Some(FourCCVideo::BGRA),
            NDIColorFormat::RGBX_RGBA | NDIColorFormat::UYVY_RGBA => Some(FourCCVideo::RGBA),
            _ => None,
        }
    }

    /// A None argument means Don't care
    pub fn from_four_cc(
        with_alpha: Option<FourCCVideo>,
        without_alpha: Option<FourCCVideo>,
    ) -> Result<Self, FromFourCCError> {
        use FourCCVideo::*;
        use NDIColorFormat::*;
        if let Some(format) = with_alpha {
            match format {
                BGRA | RGBA => {}
                BGRX | RGBX | UYVY => {
                    return Err(FromFourCCError::WrongAlphaMode {
                        format,
                        has_alpha: false,
                        expected_alpha: true,
                    });
                }
                format => {
                    return Err(FromFourCCError::UnsupportedFormat { format });
                }
            }
        }
        if let Some(format) = without_alpha {
            match format {
                BGRX | RGBX | UYVY => {}
                BGRA | RGBA => {
                    return Err(FromFourCCError::WrongAlphaMode {
                        format,
                        has_alpha: true,
                        expected_alpha: false,
                    });
                }
                format => {
                    return Err(FromFourCCError::UnsupportedFormat { format });
                }
            }
        }
        match (without_alpha, with_alpha) {
            (Some(BGRX), Some(BGRA)) => Ok(BGRX_BGRA),
            (Some(RGBX), Some(RGBA)) => Ok(RGBX_RGBA),
            (Some(UYVY), Some(BGRA)) => Ok(UYVY_BGRA),
            (Some(UYVY), Some(RGBA)) => Ok(UYVY_RGBA),
            (Some(BGRX), None) => Ok(BGRX_BGRA),
            (Some(RGBX), None) => Ok(RGBX_RGBA),
            (Some(UYVY), None) => Ok(UYVY_BGRA),
            (None, Some(BGRA)) => Ok(BGRX_BGRA),
            (None, Some(RGBA)) => Ok(RGBX_RGBA),
            (None, None) => Ok(Fastest),
            _ => Err(FromFourCCError::UnsupportedCombination),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromFourCCError {
    UnsupportedFormat {
        format: FourCCVideo,
    },
    WrongAlphaMode {
        format: FourCCVideo,
        has_alpha: bool,
        expected_alpha: bool,
    },
    UnsupportedCombination,
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum NDIBandwidthMode {
    #[default]
    Default = bindings::NDIlib_recv_bandwidth_e_NDIlib_recv_bandwidth_highest,
    Preview = bindings::NDIlib_recv_bandwidth_e_NDIlib_recv_bandwidth_lowest,
    AudioOnly = bindings::NDIlib_recv_bandwidth_e_NDIlib_recv_bandwidth_audio_only,
    MetadataOnly = bindings::NDIlib_recv_bandwidth_e_NDIlib_recv_bandwidth_metadata_only,
}

impl NDIBandwidthMode {
    pub fn to_ffi(self) -> NDIlib_recv_bandwidth_e {
        self.into()
    }

    pub fn from_ffi(value: NDIlib_recv_bandwidth_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum NDIFieldedFrameMode {
    #[default]
    Progressive = bindings::NDIlib_frame_format_type_e_NDIlib_frame_format_type_progressive,
    /// This is a frame of video that is comprised of two fields.
    /// The upper field comes first, and the lower comes second
    Interleaved = bindings::NDIlib_frame_format_type_e_NDIlib_frame_format_type_interleaved,
    /// This is an individual field 0 from a fielded video frame. This is the first temporal, upper field
    Field0 = bindings::NDIlib_frame_format_type_e_NDIlib_frame_format_type_field_0,
    /// This is an individual field 1 from a fielded video frame. This is the second temporal, lower field
    Field1 = bindings::NDIlib_frame_format_type_e_NDIlib_frame_format_type_field_1,
}

impl NDIFieldedFrameMode {
    pub fn to_ffi(self) -> NDIlib_frame_format_type_e {
        self.into()
    }

    pub fn from_ffi(value: NDIlib_frame_format_type_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }
}
