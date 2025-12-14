use std::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    /// # Panics
    ///
    /// Panics if the given resolution is not safe ([Resolution::is_safe])
    pub fn new(x: usize, y: usize) -> Self {
        let res = Resolution { x, y };
        res.check_is_safe();
        res
    }

    pub fn try_new(x: usize, y: usize) -> Option<Self> {
        if Resolution::is_safe(x, y) {
            Some(Resolution { x, y })
        } else {
            None
        }
    }

    pub const fn new_const(x: usize, y: usize) -> Self {
        assert!(
            Resolution::is_safe(x, y),
            "Resolution is not safe (more info is not available in the const version of the constructor)"
        );
        Resolution { x, y }
    }

    pub fn from_i32(x: i32, y: i32) -> Self {
        let res = Resolution {
            x: x.try_into()
                .expect("Invalid x-resolution, failed to cast to usize (x = {x})"),
            y: y.try_into()
                .expect("Invalid y-resolution, failed to cast to usize (y = {y})"),
        };
        res.check_is_safe();
        res
    }

    pub const fn to_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.x as f64 / self.y as f64
    }

    pub const fn pixels(&self) -> usize {
        // Invariant: this type cannot be constructed with unsafe values from the outside
        self.x * self.y
    }

    fn check_is_safe(&self) {
        assert!(
            Resolution::is_safe(self.x, self.x),
            "Resolution is not safe: {}x{}",
            self.x,
            self.y
        );
    }

    /// Checks if the resolution is safe to handle
    ///
    /// A resolution is considered unsafe if
    /// - one component is zero
    /// - `width * height * 4 color components * 16bit` exceeds i32::MAX
    /// - width is not divisible by 2
    pub const fn is_safe(x: usize, y: usize) -> bool {
        const MAX_SAFE_VALUE: usize = i32::MAX as usize;
        const MAX_PIXEL_BYTES: usize = 8; // 4 components x 16bit
        const MAX_SAFE_PIXELS: usize = MAX_SAFE_VALUE / MAX_PIXEL_BYTES;

        if x == 0 || y == 0 {
            return false;
        }

        // https://docs.ndi.video/all/developing-with-ndi/sdk/frame-types#video-frames-ndilib_video_frame_v2_t
        // >  Note that, because data is internally all considered in 4:2:2 formats, image width values should be divisible by two.
        if x & 0b1 != 0 {
            return false;
        }

        if x >= MAX_SAFE_VALUE || y >= MAX_SAFE_VALUE {
            return false;
        }

        if let Some(area) = usize::checked_mul(x, y) {
            area < MAX_SAFE_PIXELS
        } else {
            false
        }
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

impl Debug for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resolution({}x{})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_safe() {
        _ = Resolution::new(1920, 1080);
    }

    #[test]
    fn test_resolution_unsafe() {
        fn assert_unsafe(x: usize, y: usize) {
            assert!(
                !Resolution::is_safe(x, y),
                "Resolution {}x{} should be unsafe",
                x,
                y
            );
        }

        // zero dimensions
        assert_unsafe(0, 1080);
        assert_unsafe(1920, 0);

        // mul overflow
        assert_unsafe(i32::MAX as usize, 2);
    }
}
