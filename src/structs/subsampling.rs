use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Subsampling {
    pub x_ref: u8,
    pub x_samples: u8,
    pub x2_samples: u8,
}

impl Display for Subsampling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.x_ref, self.x_samples, self.x2_samples)
    }
}

impl Debug for Subsampling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Subsampling({}:{}:{})",
            self.x_ref, self.x_samples, self.x2_samples
        )
    }
}

impl Subsampling {
    pub fn new(x_ref: u8, x_samples: u8, x2_samples: u8) -> Self {
        Subsampling {
            x_ref,
            x_samples,
            x2_samples,
        }
    }

    pub fn is_subsampled(&self) -> bool {
        self.x_ref != self.x_samples || self.x_ref != self.x2_samples
    }

    pub fn is_regular(&self) -> bool {
        if self.x_ref == 0 {
            return false;
        }

        if self.x_samples > self.x_ref || self.x2_samples > self.x_samples {
            return false;
        }

        if self.x_ref % self.x_samples != 0 {
            return false;
        }

        if self.x2_samples % self.x_samples != 0 {
            return false;
        }

        true
    }

    pub fn x_grouping(&self) -> u8 {
        assert!(
            self.is_regular(),
            "Subsampling must be regular to get x grouping"
        );

        self.x_samples / self.x_ref
    }

    pub fn y_grouping(&self) -> u8 {
        assert!(
            self.is_regular(),
            "Subsampling must be regular to get y grouping"
        );

        self.x2_samples / self.x_samples
    }
}

impl Default for Subsampling {
    fn default() -> Self {
        Self {
            x_ref: 4,
            x_samples: 4,
            x2_samples: 4,
        }
    }
}
