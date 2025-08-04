pub mod buffer_info;
pub mod resolution;
pub mod subsampling;
pub mod tally;

pub use buffer_info::*;
pub use resolution::*;
pub use subsampling::*;
pub use tally::*;

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
