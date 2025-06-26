use std::fmt::Display;

use crate::bindings;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tally {
    pub(crate) program: bool,
    pub(crate) preview: bool,
}

impl Tally {
    pub fn on_program(&self) -> bool {
        self.program
    }

    pub fn on_preview(&self) -> bool {
        self.preview
    }

    pub fn is_shown(&self) -> bool {
        self.program || self.preview
    }

    pub(crate) fn from_ffi(tally: &bindings::NDIlib_tally_t) -> Self {
        Tally {
            program: tally.on_program,
            preview: tally.on_preview,
        }
    }

    pub(crate) fn to_ffi(&self) -> bindings::NDIlib_tally_t {
        bindings::NDIlib_tally_t {
            on_program: self.program,
            on_preview: self.preview,
        }
    }
}

pub struct BlockingUpdate<T> {
    pub value: T,
    pub(crate) changed: bool,
}

impl<T> BlockingUpdate<T> {
    pub(crate) fn new(value: T, changed: bool) -> Self {
        BlockingUpdate { value, changed }
    }
    pub fn timeout_reached(&self) -> bool {
        !self.changed
    }

    pub fn value_updated(&self) -> bool {
        self.changed
    }
}
