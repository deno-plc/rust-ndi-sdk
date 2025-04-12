use std::{ffi::CString, fmt::Debug};

use static_assertions::assert_impl_all;

use crate::{bindings, structs::NDISourceLike};

#[derive(Debug, Clone)]
pub struct NDIRouterBuilder {
    name: String,
}
assert_impl_all!(NDIRouterBuilder: Send, Sync);

impl NDIRouterBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn build(&self) -> Result<NDIRouter, String> {
        let name = CString::new(self.name.clone()).unwrap();
        let options = bindings::NDIlib_routing_create_t {
            p_ndi_name: name.as_ptr(),
            p_groups: std::ptr::null(),
        };
        let handle = unsafe { bindings::NDIlib_routing_create(&options) };

        if handle.is_null() {
            return Err("Failed to create NDI router".to_string());
        }

        Ok(NDIRouter { handle })
    }
}

#[derive(Debug)]
pub struct NDIRouter {
    handle: bindings::NDIlib_routing_instance_t,
}
unsafe impl Send for NDIRouter {}
unsafe impl Sync for NDIRouter {}

impl NDIRouter {
    pub fn switch(&mut self, source: impl NDISourceLike) -> Option<()> {
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
}

impl Drop for NDIRouter {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_routing_destroy(self.handle) };
    }
}
