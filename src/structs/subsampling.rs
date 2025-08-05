use std::fmt::Debug;
use std::fmt::Display;

/// Describes the chroma subsampling system
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
    pub const fn new(x_ref: u8, x_samples: u8, x2_samples: u8) -> Self {
        Subsampling {
            x_ref,
            x_samples,
            x2_samples,
        }
    }

    pub const fn none() -> Self {
        Subsampling {
            x_ref: 4,
            x_samples: 4,
            x2_samples: 4,
        }
    }

    pub const fn is_subsampled(&self) -> bool {
        self.x_ref != self.x_samples || self.x_ref != self.x2_samples
    }

    pub const fn is_regular(&self) -> bool {
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

        self.x_ref / self.x_samples
    }

    pub fn y_grouping(&self) -> u8 {
        assert!(
            self.is_regular(),
            "Subsampling must be regular to get y grouping"
        );

        if self.x2_samples == 0 {
            2
        } else if self.x_samples == self.x2_samples {
            1
        } else {
            panic!(
                "Subsampling is not regular, cannot determine y grouping: {}",
                self
            )
        }
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

#[cfg(test)]
mod tests {
    use super::Subsampling;

    #[test]
    fn no_subsampling() {
        let not_subsampled = Subsampling::new(4, 4, 4);
        assert!(!not_subsampled.is_subsampled());
        assert!(not_subsampled.is_regular());
        assert_eq!(not_subsampled.x_grouping(), 1);
        assert_eq!(not_subsampled.y_grouping(), 1);
    }

    #[test]
    fn subsampling_4_2_2() {
        let subsampled = Subsampling::new(4, 2, 2);
        assert!(subsampled.is_subsampled());
        assert!(subsampled.is_regular());
        assert_eq!(subsampled.x_grouping(), 2);
        assert_eq!(subsampled.y_grouping(), 1);
    }

    #[test]
    fn subsampling_4_2_0() {
        let subsampled = Subsampling::new(4, 2, 0);
        assert!(subsampled.is_subsampled());
        assert!(subsampled.is_regular());
        assert_eq!(subsampled.x_grouping(), 2);
        assert_eq!(subsampled.y_grouping(), 2);
    }

    #[test]
    fn subsampling_4_1_1() {
        let subsampled = Subsampling::new(4, 1, 1);
        assert!(subsampled.is_subsampled());
        assert!(subsampled.is_regular());
        assert_eq!(subsampled.x_grouping(), 4);
        assert_eq!(subsampled.y_grouping(), 1);
    }

    #[test]
    fn subsampling_4_4_0() {
        let subsampled = Subsampling::new(4, 4, 0);
        assert!(subsampled.is_subsampled());
        assert!(subsampled.is_regular());
        assert_eq!(subsampled.x_grouping(), 1);
        assert_eq!(subsampled.y_grouping(), 2);
    }

    #[test]
    fn irregular_subsampling() {
        let irregular = Subsampling::new(4, 3, 2);
        assert!(irregular.is_subsampled());
        assert!(!irregular.is_regular());
    }

    #[test]
    fn zero_reference() {
        let zero_ref = Subsampling::new(0, 0, 0);
        assert!(!zero_ref.is_regular());
    }
}
