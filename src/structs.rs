use std::ffi::CStr;

use crate::bindings;

pub trait NDISourceLike {
    fn to_descriptor(&self) -> *const bindings::NDIlib_source_t;
}

/// wrapper for `NDIlib_source_t`
/// This is a short-lived/reference source descriptor
pub struct NDISourceRef<'a> {
    name: &'a CStr,
    raw: &'a bindings::NDIlib_source_t,
}

impl<'a> NDISourceRef<'a> {
    pub(crate) fn from(source_t: &'a bindings::NDIlib_source_t) -> Self {
        let name = unsafe { CStr::from_ptr(source_t.p_ndi_name) };
        NDISourceRef {
            name,
            raw: source_t,
        }
    }
    pub fn name(&'a self) -> &'a CStr {
        self.name
    }
    pub fn to_owned(&self) -> NDISource {
        NDISource {
            name: self.name.to_owned().into_string().unwrap(),
            raw: *self.raw,
        }
    }
}
impl NDISourceLike for &NDISourceRef<'_> {
    fn to_descriptor(&self) -> *const bindings::NDIlib_source_t {
        self.raw
    }
}

/// long-lived/owned source descriptor
pub struct NDISource {
    name: String,
    raw: bindings::NDIlib_source_t,
}
impl NDISource {
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl NDISourceLike for &NDISource {
    fn to_descriptor(&self) -> *const bindings::NDIlib_source_t {
        &self.raw
    }
}
impl NDISourceLike for NDISource {
    fn to_descriptor(&self) -> *const bindings::NDIlib_source_t {
        &self.raw
    }
}
