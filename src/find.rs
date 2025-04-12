use core::slice;
use std::time::Duration;

use crate::{bindings, structs::NDISourceRef};

/// wrapper for `NDIlib_find_create_t`
#[derive(Debug, Clone)]
pub struct NDISourceFinderBuilder {
    show_local_sources: bool,
}

impl Default for NDISourceFinderBuilder {
    fn default() -> Self {
        Self {
            show_local_sources: true,
        }
    }
}

impl NDISourceFinderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_local_sources(mut self, show: bool) -> Self {
        self.show_local_sources = show;
        self
    }

    pub fn build(self) -> Option<NDISourceFinder> {
        let options = bindings::NDIlib_find_create_t {
            show_local_sources: self.show_local_sources,
            p_groups: std::ptr::null(),
            p_extra_ips: std::ptr::null(),
        };
        let handle = unsafe { bindings::NDIlib_find_create_v2(&options) };
        if handle.is_null() {
            return None;
        } else {
            Some(NDISourceFinder { handle })
        }
    }
}

/// wrapper for `NDIlib_find_instance_t`
#[derive(Debug)]
pub struct NDISourceFinder {
    handle: bindings::NDIlib_find_instance_t,
}
unsafe impl Send for NDISourceFinder {}

impl<'a> NDISourceFinder {
    /// Returns an iterator over the sources currently known to this device
    pub fn get_source_iter(&'a mut self) -> Option<NDISourceIterator<'a>> {
        let mut num_sources = 0u32;
        let sources =
            unsafe { bindings::NDIlib_find_get_current_sources(self.handle, &mut num_sources) };

        if sources.is_null() {
            return None;
        }

        let src_slice = unsafe { slice::from_raw_parts(sources, num_sources as usize) };

        Some(NDISourceIterator {
            iter: src_slice.into_iter(),
        })
    }

    pub fn blocking_wait_for_change(&mut self, timeout: Duration) -> bool {
        unsafe {
            bindings::NDIlib_find_wait_for_sources(
                self.handle,
                timeout.as_millis().try_into().unwrap_or(u32::MAX),
            )
        }
    }
}

impl Drop for NDISourceFinder {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_find_destroy(self.handle) }
    }
}

pub struct NDISourceIterator<'a> {
    iter: slice::Iter<'a, bindings::NDIlib_source_t>,
}
impl<'a> Iterator for NDISourceIterator<'a> {
    type Item = NDISourceRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(source) = self.iter.next() {
            Some(NDISourceRef::from(source))
        } else {
            None
        }
    }
}
