use std::ffi::CString;

use crate::{bindings, structs::NDISourceLike};

pub struct NDIRouterBuilder {
    name: String,
}

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

pub struct NDIRouter {
    handle: bindings::NDIlib_routing_instance_t,
}

impl NDIRouter {
    pub fn switch(&mut self, source: impl NDISourceLike) -> Option<()> {
        let source_ptr = source.to_descriptor();
        let result = unsafe { bindings::NDIlib_routing_change(self.handle, source_ptr) };
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
