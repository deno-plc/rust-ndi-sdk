//! Tally indicator

use crate::bindings;

/// Tally indicator state
///
/// C equivalent: `NDIlib_tally_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tally {
    pub program: bool,
    pub preview: bool,
}

impl Tally {
    pub fn new(program: bool, preview: bool) -> Self {
        Tally { program, preview }
    }

    pub fn on_program(&self) -> bool {
        self.program
    }

    pub fn set_program(&mut self, on: bool) {
        self.program = on;
    }

    pub fn on_preview(&self) -> bool {
        self.preview
    }

    pub fn set_preview(&mut self, on: bool) {
        self.preview = on;
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

    pub(crate) fn to_ffi(self) -> bindings::NDIlib_tally_t {
        bindings::NDIlib_tally_t {
            on_program: self.program,
            on_preview: self.preview,
        }
    }
}
