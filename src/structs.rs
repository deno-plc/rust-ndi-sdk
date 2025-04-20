use std::{
    ffi::{CStr, CString},
    fmt::{Debug, Display},
    ptr,
};

use static_assertions::assert_impl_all;

use crate::bindings;

pub trait NDISourceLike: Debug {
    /// This method builds a source descriptor for the NDI library that is valid for the duration of the closure call
    ///
    /// SAFETY: The pointer passed to the closure is valid for the duration of the closure call.
    /// SAFETY: This might be a null pointer if it is generated from `NullPtrSource` or `Option<NDISourceLike>::None`.
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NullPtrSource;

impl NDISourceLike for NullPtrSource {
    /// SAFETY: This is always a null pointer
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        f(ptr::null())
    }
}

impl<S: NDISourceLike> NDISourceLike for Option<S> {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        if let Some(source) = self {
            source.with_descriptor(f)
        } else {
            NullPtrSource.with_descriptor(f)
        }
    }
}

/// wrapper for `NDIlib_source_t`
/// This is a short-lived/reference source descriptor
pub struct NDISourceRef<'a> {
    name: &'a CStr,
    raw: &'a bindings::NDIlib_source_t,
}
// needs to be evaluated
// unsafe impl Send for NDISourceRef<'_> {}

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
        let descriptor_anon_1 = self.raw.__bindgen_anon_1;
        let descriptor_anon_1: *const ::std::os::raw::c_char =
            unsafe { descriptor_anon_1.p_url_address };
        let descriptor_anon_1 = if descriptor_anon_1.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(descriptor_anon_1).to_owned() })
        };

        let name = self.name.to_owned();

        NDISource {
            name: name.clone().into_string().unwrap(),
            name_c: name,
            descriptor_anon_1,
        }
    }
}
impl NDISourceLike for &NDISourceRef<'_> {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        f(self.raw)
    }
}
impl std::fmt::Debug for NDISourceRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let descriptor_anon_1 = self.raw.__bindgen_anon_1;
        let descriptor_anon_1: *const ::std::os::raw::c_char =
            unsafe { descriptor_anon_1.p_url_address };

        let mut dbg = f.debug_struct("NDISourceRef");

        dbg.field("name", &self.name.to_str());

        if !descriptor_anon_1.is_null() {
            let descriptor = unsafe { CStr::from_ptr(descriptor_anon_1) };
            dbg.field("raw_src", &descriptor);
        } else {
            dbg.field("raw_src", &"null");
        }

        dbg.finish()
    }
}

/// long-lived/owned source descriptor
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct NDISource {
    name: String,
    name_c: CString,
    descriptor_anon_1: Option<CString>,
}

assert_impl_all!(NDISource: Send, Sync);

impl NDISource {
    pub fn from_name(name: &str) -> Self {
        let name_c = CString::new(name).unwrap();
        NDISource {
            name: name.to_owned(),
            name_c,
            descriptor_anon_1: None,
        }
    }
}

impl NDISource {
    pub fn name(&self) -> &str {
        &self.name.as_str()
    }
}

impl NDISourceLike for NDISource {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        let descriptor = bindings::NDIlib_source_t {
            p_ndi_name: self.name_c.as_ptr(),
            __bindgen_anon_1: bindings::NDIlib_source_t__bindgen_ty_1 {
                p_url_address: self
                    .descriptor_anon_1
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
            },
        };
        f(&descriptor)
    }
}

impl NDISourceLike for &NDISource {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        (*self).with_descriptor(f)
    }
}

impl std::fmt::Debug for NDISource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NDISource")
            .field("name", &self.name)
            .field("raw_src", &self.descriptor_anon_1)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    pub fn new(x: usize, y: usize) -> Self {
        Resolution { x, y }
    }

    pub fn pixels(&self) -> usize {
        self.x * self.y
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}
