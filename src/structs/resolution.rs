use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    pub fn new(x: usize, y: usize) -> Self {
        let res = Resolution { x, y };
        #[cfg(any(debug_assertions, feature = "strict_assertions"))]
        assert!(res.is_safe(), "Resolution is not safe: {}x{}", res.x, res.y);
        res
    }

    pub const fn new_const(x: usize, y: usize) -> Self {
        let res = Resolution { x, y };
        #[cfg(any(debug_assertions, feature = "strict_assertions"))]
        assert!(
            res.is_safe(),
            "Resolution is not safe (more info is not available in the const version of the constructor)"
        );
        res
    }

    pub fn from_i32(x: i32, y: i32) -> Self {
        let res = Resolution {
            x: x.try_into()
                .expect("Invalid x-resolution, failed to cast to usize"),
            y: y.try_into()
                .expect("Invalid y-resolution, failed to cast to usize"),
        };
        #[cfg(any(debug_assertions, feature = "strict_assertions"))]
        assert!(res.is_safe(), "Resolution is not safe: {}x{}", res.x, res.y);
        res
    }

    pub const fn to_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub const fn pixels(&self) -> usize {
        debug_assert!(self.is_safe(), "Resolution is not safe");
        self.x * self.y
    }

    pub const fn is_safe(&self) -> bool {
        if self.x == 0 || self.y == 0 {
            return false;
        }

        const MAX_SAFE_VALUE: usize = i32::MAX as usize;

        if self.x > MAX_SAFE_VALUE || self.y > MAX_SAFE_VALUE {
            return false;
        }

        const MAX_SAFE_PIXELS: usize = MAX_SAFE_VALUE / 8; // 4 components x 16bit

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
