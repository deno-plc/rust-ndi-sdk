use std::{error::Error, ffi::CString, time::Duration};

pub(crate) fn duration_to_ms(dur: Duration) -> u32 {
    dur.as_millis().try_into().unwrap_or(u32::MAX)
}

/// The total length of an NDI source name should be limited to 253 characters. The following characters
/// are considered invalid: `\ / : * ? " < > |`. If any of these characters are found in the name, they will
/// be replaced with a space. These characters are reserved according to Windows file system naming conventions
pub fn validate_source_name(name: &str) -> Result<CString, SourceNameError> {
    let name = CString::new(name).map_err(SourceNameError::NulError)?;
    if name.count_bytes() >= 253 {
        Err(SourceNameError::TooLong)
    } else {
        Ok(name)
    }
}

/// see [validate_source_name] for more information
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceNameError {
    /// The input name contained Nul characters
    NulError(std::ffi::NulError),
    /// The total length of an NDI source name should be limited to 253 characters
    TooLong,
}

impl std::fmt::Display for SourceNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NulError(nul_error) => write!(
                f,
                "Source name contains null characters at index {}",
                nul_error.nul_position()
            ),
            Self::TooLong => {
                f.write_str("Source name is too long, a maximum of 253 bytes are supported by NDI")
            }
        }
    }
}

impl Error for SourceNameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NulError(nul_error) => Some(nul_error),
            Self::TooLong => None,
        }
    }
}
