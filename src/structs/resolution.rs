use std::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    pub fn new(x: usize, y: usize) -> Self {
        let res = Resolution { x, y };
        assert!(res.is_safe(), "Resolution is not safe: {}x{}", res.x, res.y);
        res
    }

    pub const fn new_const(x: usize, y: usize) -> Self {
        let res = Resolution { x, y };
        assert!(
            res.is_safe(),
            "Resolution is not safe (more info is not available in the const version of the constructor)"
        );
        res
    }

    pub fn from_i32(x: i32, y: i32) -> Self {
        let res = Resolution {
            x: x.try_into()
                .expect("Invalid x-resolution, failed to cast to usize (x = {x})"),
            y: y.try_into()
                .expect("Invalid y-resolution, failed to cast to usize (y = {y})"),
        };
        assert!(res.is_safe(), "Resolution is not safe: {}x{}", res.x, res.y);
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

    /// Checks if the resolution is safe to handle
    ///
    /// A resolution is considered unsafe if
    /// - one component is zero
    /// - `width * height * 4 color components * 16bit` exceeds i32::MAX
    /// - width is not divisible by 2
    pub const fn is_safe(&self) -> bool {
        const MAX_SAFE_VALUE: usize = i32::MAX as usize;
        const MAX_PIXEL_BYTES: usize = 8; // 4 components x 16bit
        const MAX_SAFE_PIXELS: usize = MAX_SAFE_VALUE / MAX_PIXEL_BYTES;

        if self.x == 0 || self.y == 0 {
            return false;
        }

        // https://docs.ndi.video/all/developing-with-ndi/sdk/frame-types#video-frames-ndilib_video_frame_v2_t
        // >  Note that, because data is internally all considered in 4:2:2 formats, image width values should be divisible by two.
        if self.x & 0b1 != 0 {
            return false;
        }

        if self.x >= MAX_SAFE_VALUE || self.y >= MAX_SAFE_VALUE {
            return false;
        }

        if let Some(area) = usize::checked_mul(self.x, self.y) {
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
        let res = Resolution::new(1920, 1080);
        assert!(res.is_safe());
    }

    // if we would use the standard constructor, it would panic before we can test anything
    fn new_unsafe_resolution(x: usize, y: usize) -> Resolution {
        Resolution { x, y }
    }

    #[test]
    fn test_resolution_unsafe() {
        fn assert_unsafe(x: usize, y: usize) {
            let res = new_unsafe_resolution(x, y);
            assert!(!res.is_safe(), "Resolution {}x{} should be unsafe", x, y);
        }

        // zero dimensions
        assert_unsafe(0, 1080);
        assert_unsafe(1920, 0);

        // mul overflow
        assert_unsafe(i32::MAX as usize, 2);
    }
}
