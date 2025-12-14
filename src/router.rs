//! NDI Router
//!
//! Using NDI routing, you can create an output on a machine that looks just like a ‘real’ video source to all remote systems.
//! However, rather than producing actual video frames, it directs sources watching this output to receive video from a different location.
//!
//! NDI routing does not actually transfer any data through the computer hosting the routing source; it merely instructs receivers to look at another location when they wish to receive data from the router.
//! Thus, a computer can act as a router exposing potentially hundreds of routing sources to the network – without any bandwidth overhead.
//! This facility can be used for large-scale dynamic switching of sources at a network level.
//!
//! <https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-routing>

use std::{ffi::CString, fmt::Debug};

use static_assertions::assert_impl_all;

use crate::{
    bindings,
    source::{NDISourceLike, NDISourceRef},
    util::{SourceNameError, validate_source_name},
};

/// Builder for [NDIRouter]
#[derive(Debug, Clone)]
pub struct NDIRouterBuilder {
    name: CString,
}
assert_impl_all!(NDIRouterBuilder: Send, Sync);

impl NDIRouterBuilder {
    /// Creates a new [NDIRouterBuilder] with the given source name
    ///
    /// The total length of an NDI source name should be limited to 253 characters. The following characters
    /// are considered invalid: \ / : * ? " < > |. If any of these characters are found in the name, they will
    /// be replaced with a space. These characters are reserved according to Windows file system naming conventions
    pub fn new(name: &str) -> Result<Self, SourceNameError> {
        Ok(Self {
            name: validate_source_name(name)?,
        })
    }

    pub fn build(self) -> Result<NDIRouter, NDIRouterBuilderError> {
        let options = bindings::NDIlib_routing_create_t {
            // We only need the name for the constructor
            p_ndi_name: self.name.as_ptr(),
            p_groups: std::ptr::null(),
        };
        let handle = unsafe { bindings::NDIlib_routing_create(&options) };

        if handle.is_null() {
            return Err(NDIRouterBuilderError::CreationFailed);
        }

        Ok(NDIRouter { handle })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NDIRouterBuilderError {
    CreationFailed,
}

/// NDI Router
///
/// For more information see module docs
#[derive(Debug)]
pub struct NDIRouter {
    handle: bindings::NDIlib_routing_instance_t,
}
unsafe impl Send for NDIRouter {}
unsafe impl Sync for NDIRouter {}

impl NDIRouter {
    pub fn switch(&mut self, source: &impl NDISourceLike) -> Option<()> {
        let mut result = false;
        source.with_descriptor(|source_ptr| {
            result = unsafe { bindings::NDIlib_routing_change(self.handle, source_ptr) };
        });
        if result { Some(()) } else { None }
    }

    pub fn switch_clear(&mut self) -> Option<()> {
        let result = unsafe { bindings::NDIlib_routing_clear(self.handle) };
        if result { Some(()) } else { None }
    }

    /// Get the source descriptor that is needed to connect to this router source
    pub fn get_source(&self) -> NDISourceRef<'_> {
        // SDK Docs: Retrieve the source information for the given router instance. This pointer is valid until NDIlib_routing_destroy is called.
        let source = unsafe { bindings::NDIlib_routing_get_source_name(self.handle) };

        unsafe {
            NDISourceRef::from(
                *source
                    .as_ref()
                    .expect("[Fatal FFI Error] NDI SDK returned nullptr for source descriptor"),
            )
        }
    }
}

impl Drop for NDIRouter {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_routing_destroy(self.handle) };
    }
}
