use std::time::{Duration, SystemTime};

use static_assertions::{const_assert, const_assert_eq};

use crate::bindings;

/// This is the timecode of this frame in 100 ns intervals.
/// This is generally not used internally by the SDK but is passed through to applications which may interpret it as they wish.
/// When sending data, a value of [NDITime::SYNTHESIZE] (C: `NDIlib_send_timecode_synthesize`) can be specified (and should be the default), the operation of this value is documented in the sending section of this documentation.
///
/// [NDITime::SYNTHESIZE] will yield UTC time in 100 ns intervals since the Unix Time Epoch 1/1/1970 00:00.
/// When interpreting this timecode, a receiving application may choose to localize the time of day based on time zone offset, which can optionally be communicated by the sender in connection metadata.
///
/// Since the timecode is stored in UTC within NDI, communicating timecode time of day for non-UTC time zones requires a translation.
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

    /// **This API is unstable**
    ///
    /// Converts this timestamp to [SystemTime]
    pub fn to_utc(self) -> Option<SystemTime> {
        fn to_duration(time: u64) -> Duration {
            let secs = time / 10_000_000;
            let sub_100ns = time % 10_000_000;

            const_assert!(10_000_000 < i32::MAX);

            let nanos = (sub_100ns * 100) as u32;

            Duration::new(secs, nanos)
        }

        if self.is_default() {
            None
        } else {
            Some(if self.0 >= 0 {
                SystemTime::UNIX_EPOCH + to_duration(self.0 as u64)
            } else {
                SystemTime::UNIX_EPOCH - to_duration(self.0.saturating_abs() as u64)
            })
        }
    }

    pub fn is_default(self) -> bool {
        self.0 == NDI_TIME_DEFAULT
    }

    pub const UNDEFINED: Self = Self(NDI_TIME_DEFAULT);
    /// Advises the SDK to automatically generate a timecode for this frame from the system clock
    pub const SYNTHESIZE: Self = Self(NDI_TIME_DEFAULT);
}
