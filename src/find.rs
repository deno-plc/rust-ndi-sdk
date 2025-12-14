//! NDI source finder
//!
//! This is provided to locate sources available on the network and is normally used in conjunction with **NDI-Receive**.
//! Internally, it uses a cross-process P2P mDNS implementation to locate sources on the network.
//! (It commonly takes a few seconds to locate all the sources available since this requires other running machines to send response messages.)
//!
//! Although discovery uses mDNS, the client is entirely self-contained; Bonjour (etc.) is not required.
//! mDNS is a P2P system that exchanges located network sources and provides a highly robust and bandwidth-efficient way to perform discovery on a local network.
//!
//! On mDNS initialization (often done using the **NDI-FIND** SDK), a few seconds might elapse before all sources on the network are located.
//! Be aware that some network routers might block mDNS traffic between network segments.
//!
//! <https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-find>

use core::slice;
use std::time::Duration;

use crate::{
    bindings, blocking_update::BlockingUpdate, source::NDISourceRef, util::duration_to_ms,
};

/// Builder for [NDISourceFinder]
#[non_exhaustive]
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

    /// Show sources from this device (default: true)
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
            None
        } else {
            Some(NDISourceFinder { handle })
        }
    }
}

/// NDI Source finder
///
/// For more information see module docs
///
/// C equivalent: `NDIlib_find_instance_t`
#[derive(Debug)]
pub struct NDISourceFinder {
    handle: bindings::NDIlib_find_instance_t,
}
unsafe impl Send for NDISourceFinder {}

impl<'a> NDISourceFinder {
    /// Returns an iterator over the sources currently known to this device[^note]
    ///
    /// [^note]: Yes this really needs an `&mut` as calling `NDIlib_find_get_current_sources` invalidates all previous iterators
    pub fn get_source_iter(&'a mut self) -> Option<impl Iterator<Item = NDISourceRef<'a>>> {
        let mut num_sources = 0u32;

        // SDK Docs: The pointer returned by NDIlib_find_get_current_sources is owned by the finder instance, so there is no reason to free it. It will be retained until the next call to NDIlib_find_get_current_sources, or until the NDIlib_find_destroy function is destroyed.

        // We borrow self &mut, and therefore the iterator ('a) can only live as long we have the only borrow to the instance
        let sources =
            unsafe { bindings::NDIlib_find_get_current_sources(self.handle, &mut num_sources) };

        if sources.is_null() {
            return None;
        }

        let src_slice = unsafe { slice::from_raw_parts(sources, num_sources as usize) };

        Some(
            src_slice
                .iter()
                .map(|src| unsafe { NDISourceRef::from(*src) }),
        )
    }

    /// Blocks until the source list changes or the timeout is reached
    pub fn wait_for_change(&mut self, timeout: Duration) -> BlockingUpdate<()> {
        let changed =
            unsafe { bindings::NDIlib_find_wait_for_sources(self.handle, duration_to_ms(timeout)) };

        BlockingUpdate::new((), changed)
    }
}

impl Drop for NDISourceFinder {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_find_destroy(self.handle) }
    }
}
