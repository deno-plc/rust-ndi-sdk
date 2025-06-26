use static_assertions::const_assert_eq;

use crate::bindings;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NDITime(i64);

const NDI_TIME_DEFAULT: i64 = i64::MAX;

const_assert_eq!(NDI_TIME_DEFAULT, bindings::NDIlib_recv_timestamp_undefined);
const_assert_eq!(NDI_TIME_DEFAULT, bindings::NDIlib_send_timecode_synthesize);

impl Default for NDITime {
    fn default() -> Self {
        Self(NDI_TIME_DEFAULT)
    }
}

impl NDITime {
    #[inline]
    pub fn to_ffi(self) -> i64 {
        self.0
    }

    #[inline]
    pub fn from_ffi(time: i64) -> Self {
        Self(time)
    }

    pub fn is_default(self) -> bool {
        self.0 == NDI_TIME_DEFAULT
    }

    pub const UNDEFINED: Self = Self(NDI_TIME_DEFAULT);
    pub const SYNTHESIZE: Self = Self(NDI_TIME_DEFAULT);
}
