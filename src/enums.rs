use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    bindings::{
        self, NDIlib_frame_format_type_e, NDIlib_recv_bandwidth_e, NDIlib_recv_color_format_e,
    },
    four_cc::FourCCVideo,
};

/// C equivalent: `NDIlib_recv_color_format_e`
/// This enum describes the preferred color format for receiving video frames.
/// If you can handle all color formats you should use `Fastest`
#[repr(i32)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum NDIPreferredColorFormat {
    #[default]
    Fastest = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_fastest,
    Best = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_best,

    BGRX_BGRA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_BGRX_BGRA,
    UYVY_BGRA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_UYVY_BGRA,
    RGBX_RGBA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_RGBX_RGBA,
    UYVY_RGBA = bindings::NDIlib_recv_color_format_e_NDIlib_recv_color_format_UYVY_RGBA,
}

impl NDIPreferredColorFormat {
    pub fn to_ffi(self) -> NDIlib_recv_color_format_e {
        self.into()
    }

    pub fn from_ffi(value: NDIlib_recv_color_format_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }

    /// Returns the FourCC type that will be used for frames without alpha channel.
    /// None indicates that the resulting FourCC type cannot statically be known
    pub const fn without_alpha_four_cc(self) -> Option<FourCCVideo> {
        match self {
            NDIPreferredColorFormat::BGRX_BGRA => Some(FourCCVideo::BGRX),
            NDIPreferredColorFormat::RGBX_RGBA => Some(FourCCVideo::RGBX),
            NDIPreferredColorFormat::UYVY_BGRA | NDIPreferredColorFormat::UYVY_RGBA => {
                Some(FourCCVideo::UYVY)
            }
            _ => None,
        }
    }

    /// Returns the FourCC type that will be used for frames with alpha channel.
    /// None indicates that the resulting FourCC type cannot statically be known
    pub const fn with_alpha_four_cc(self) -> Option<FourCCVideo> {
        match self {
            NDIPreferredColorFormat::BGRX_BGRA | NDIPreferredColorFormat::UYVY_BGRA => {
                Some(FourCCVideo::BGRA)
            }
            NDIPreferredColorFormat::RGBX_RGBA | NDIPreferredColorFormat::UYVY_RGBA => {
                Some(FourCCVideo::RGBA)
            }
            _ => None,
        }
    }

    /// Try to create a preferred color format from the given FourCC types.
    /// only a few combinations are allowed by the SDK, in case of an unsupported combination this will return Err(FromFourCCError::UnsupportedCombination).
    /// A None argument means Don't care
    pub fn from_four_cc(
        with_alpha: Option<FourCCVideo>,
        without_alpha: Option<FourCCVideo>,
    ) -> Result<Self, FromFourCCError> {
        use FourCCVideo::*;
        use NDIPreferredColorFormat::*;
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

/// Video frames can be fielded (even and odd lines are sent in separate frames), this enum
/// describes the fielding mode
#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
pub enum NDIFieldedFrameMode {
    /// Progressive (non-fielded) video frame.
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
    pub(crate) fn to_ffi(self) -> NDIlib_frame_format_type_e {
        self.into()
    }

    pub(crate) fn from_ffi(value: NDIlib_frame_format_type_e) -> Option<Self> {
        Self::try_from_primitive(value).ok()
    }

    pub const fn is_progressive(self) -> bool {
        matches!(self, NDIFieldedFrameMode::Progressive)
    }

    pub const fn is_fielded(self) -> bool {
        !self.is_progressive()
    }

    pub const fn is_single_field(self) -> bool {
        matches!(
            self,
            NDIFieldedFrameMode::Field0 | NDIFieldedFrameMode::Field1
        )
    }
}

#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NDIRecvType {
    /// A video frame was received
    Video,
    /// An audio frame was received
    Audio,
    /// A metadata frame was received
    Metadata,
    /// No frame was received, most likely because the timeout was reached.
    None,
    /// The SDK returned a frame type that is not recognized.
    Unknown,
    /// No frame was received, but the status of the connection changed.
    /// Things like the web control URL could have changed
    StatusChange,
    /// The source the receiver is connected to has changed
    SourceChange,
}
