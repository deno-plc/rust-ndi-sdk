use crate::bindings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tally {
    pub(crate) program: bool,
    pub(crate) preview: bool,
}

impl Tally {
    pub fn on_program(&self) -> bool {
        self.program
    }

    pub fn on_preview(&self) -> bool {
        self.preview
    }

    pub fn is_shown(&self) -> bool {
        self.program || self.preview
    }

    pub(crate) fn from_ffi(tally: &bindings::NDIlib_tally_t) -> Self {
        Tally {
            program: tally.on_program,
            preview: tally.on_preview,
        }
    }

    pub(crate) fn to_ffi(&self) -> bindings::NDIlib_tally_t {
        bindings::NDIlib_tally_t {
            on_program: self.program,
            on_preview: self.preview,
        }
    }
}
