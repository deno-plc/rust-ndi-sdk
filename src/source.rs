//! Source descriptors are used to get the name of local and discovered senders as well as telling a receiver unambiguously which source to connect to
//!
//! A source descriptor contains the name (exposed via `.name()`) and
//! optional internal connection info (not exposed, used to avoid creating an internal finder to resolve the name)

use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    marker::PhantomData,
    ptr,
};

use static_assertions::assert_impl_all;

use crate::{
    bindings,
    util::{SourceNameError, validate_source_name},
};

/// Everything that can be used to refer to an NDI source
///
/// # Safety
///
/// the [NDISourceLike::with_descriptor] itself is safe, but unsafe code relies on this being correct, so the whole trait has to be considered unsafe.
pub unsafe trait NDISourceLike: Debug {
    /// This method builds a source descriptor for the NDI library that is valid for the duration of the closure call
    ///
    /// # Safety
    ///
    /// The pointer passed to the closure is valid (if non-null) for the duration of the closure call.
    ///
    /// This might be a null pointer if it is generated from `NullPtrSource` or `Option<NDISourceLike>::None`.
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R;
}

/// Represents no source, equivalent to passing a nullptr in the C SDK
#[derive(Debug, Clone, Copy)]
pub(crate) struct NullPtrSource;

unsafe impl NDISourceLike for NullPtrSource {
    /// # Safety
    ///
    /// This is always a null pointer, as permitted by the trait
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        f(ptr::null())
    }
}

unsafe impl<S: NDISourceLike> NDISourceLike for Option<S> {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        if let Some(source) = self {
            source.with_descriptor(f)
        } else {
            NullPtrSource.with_descriptor(f)
        }
    }
}

/// This is a short-lived/reference source descriptor
///
/// A source descriptor contains the name (exposed via [NDISourceRef::name]) and
/// optional internal connection info (not exposed, used to avoid creating an internal finder to resolve the name)
///
/// C equivalent: `NDIlib_source_t`
#[derive(Clone)]
pub struct NDISourceRef<'a> {
    // this points to the name inside raw
    name: &'a CStr,
    raw: bindings::NDIlib_source_t,
    raw_ptrs: PhantomData<&'a CStr>,
}
// needs to be evaluated
// unsafe impl Send for NDISourceRef<'_> {}

impl<'a> NDISourceRef<'a> {
    pub(crate) unsafe fn from(source_t: bindings::NDIlib_source_t) -> Self {
        if source_t.p_ndi_name.is_null() {
            panic!("[Fatal FFI Error] NDI SDK returned nullptr for source name")
        } else {
            let name = unsafe { CStr::from_ptr(source_t.p_ndi_name) };
            NDISourceRef {
                name,
                raw: source_t,
                raw_ptrs: PhantomData,
            }
        }
    }

    /// Gets the name of the source
    pub fn name(&'a self) -> &'a CStr {
        self.name
    }

    /// Converts it to an owned [NDISource] by cloning all information
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

unsafe impl NDISourceLike for &NDISourceRef<'_> {
    fn with_descriptor<R>(&self, f: impl FnOnce(*const bindings::NDIlib_source_t) -> R) -> R {
        f(&self.raw)
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
///
/// A source descriptor contains the name (exposed via [NDISource::name]) and
/// optional internal connection info (not exposed, used to avoid creating an internal finder to resolve the name)
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct NDISource {
    name: String,
    name_c: CString,
    descriptor_anon_1: Option<CString>,
}

assert_impl_all!(NDISource: Send, Sync);

impl NDISource {
    pub fn from_name(name: &str) -> Result<Self, SourceNameError> {
        let name_c = validate_source_name(name)?;
        Ok(NDISource {
            name: name.to_owned(),
            name_c,
            descriptor_anon_1: None,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn name_c_str(&self) -> &CStr {
        &self.name_c
    }
}

unsafe impl NDISourceLike for NDISource {
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

unsafe impl NDISourceLike for &NDISource {
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
