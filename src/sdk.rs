use std::ffi::CStr;

use crate::bindings;

/// Get the version of the NDI SDK as a string.
/// This may return None if the version cannot be determined.
pub fn version() -> Option<&'static str> {
    let version_ptr = unsafe { bindings::NDIlib_version() };
    if version_ptr.is_null() {
        return None;
    }
    let version = unsafe { CStr::from_ptr(version_ptr) };
    version.to_str().ok()
}

/// Detect whether the current CPU in the system is capable of running NDILib.
/// Currently NDILib requires SSE4.2 instructions (see documentation).
pub fn cpu_supported() -> bool {
    unsafe { bindings::NDIlib_is_supported_CPU() == true }
}

/// This is not actually required, but will start the libraries which might result in a slightly better
/// performance in some cases. In general it is more "correct" to call it although it is not required.
pub fn initialize() -> Result<(), NDIInitError> {
    if unsafe { bindings::NDIlib_initialize() == true } {
        Ok(())
    } else if !cpu_supported() {
        Err(NDIInitError::UnsupportedCPU)
    } else {
        Err(NDIInitError::GenericError)
    }
}

/// This is not actually required, but will end the libraries which might result in a slightly better
/// performance in some cases. In general it is more "correct" to call it although it is not required.
/// There is no way a call it could have an adverse impact on anything (even calling destroy before
/// you've deleted all your objects).
pub fn destroy() {
    unsafe { bindings::NDIlib_destroy() }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum NDIInitError {
    UnsupportedCPU,
    GenericError,
}
