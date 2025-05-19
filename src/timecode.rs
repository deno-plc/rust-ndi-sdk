use static_assertions::const_assert_eq;

use crate::bindings;

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NDITime {
    #[default]
    Default,
    Time(i64),
}

const NDI_TIME_DEFAULT: i64 = i64::MAX;

const_assert_eq!(NDI_TIME_DEFAULT, bindings::NDIlib_recv_timestamp_undefined);
const_assert_eq!(NDI_TIME_DEFAULT, bindings::NDIlib_send_timecode_synthesize);

impl NDITime {
    pub fn to_ffi(self) -> i64 {
        match self {
            NDITime::Default => NDI_TIME_DEFAULT,
            NDITime::Time(t) => t,
        }
    }

    pub fn from_ffi(t: i64) -> Self {
        match t {
            NDI_TIME_DEFAULT => NDITime::Default,
            t => NDITime::Time(t),
        }
    }

    pub const UNDEFINED: Self = Self::Default;
    pub const SYNTHESIZE: Self = Self::Default;
}
