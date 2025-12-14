//! Helper for results of blocking operations with timeout

/// Helper for blocking operations with timeout
///
/// ```rust,no_run
/// # fn main(){
/// # use std::time::Duration;
/// # let sender = crate::ndi_sdk_sys::sender::NDISenderBuilder::new().build().unwrap();
/// let tally = sender.get_tally_update(Duration::from_secs(5));
/// tally.value; // This contains the current tally state even if nothing changed
/// if tally.value_updated(){
///     println!("Tally is now: {:?}", tally.value);
/// }
/// # }
/// ```
pub struct BlockingUpdate<T> {
    pub value: T,
    pub(crate) changed: bool,
}

impl<T> BlockingUpdate<T> {
    pub(crate) fn new(value: T, changed: bool) -> Self {
        BlockingUpdate { value, changed }
    }

    /// Indicates that the timeout was reached before the operation could finish otherwise
    pub fn timeout_reached(&self) -> bool {
        !self.changed
    }

    /// Indicates that the operation finished within the timeout
    pub fn value_updated(&self) -> bool {
        self.changed
    }
}
