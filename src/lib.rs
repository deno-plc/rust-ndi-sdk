//! Safe and ergonomic Rust bindings for the [NDI (ndi.video)](https://ndi.video/) SDK, enabling high-performance video, audio, and metadata transmission over IP networks.
//!
//! Unlike other bindings this never exposes any raw pointers and uses correct Rust types for everything (timeouts are handled as `std::time::Duration`, not `time_in_ms: u32`)
//!
//! Despite its name this is not affiliated with `ndi`, `ndi-sdk`, `ndi-sys` crates.
//!
//! ## Building
//!
//! 1. Make sure to have the [NDI SDK](https://ndi.video/for-developers/#ndi-sdk) installed
//! 2. Make sure to have Clang/LLVM installed. [see rust-bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)
//!
//! ## Docs
//!
//! This library aims to be as close to the C SDK as possible while still providing a safe abstraction.
//! In many cases the [original documentation](https://docs.ndi.video/all/developing-with-ndi/sdk) is still useful.
//! If you have already worked with it, look out for `C Equivalent: ...` comments, as they are intended to help selecting the right Rust equivalent of C structs and functions
//!
//! ## Cargo features
//!
//! ### `strict_assertions`
//!
//! This crate uses a lot of unsafe ffi bindings and therefore a lot of assertions.
//!
//! You can turn on this feature to ensure debug assertions are in place even for
//! release builds. During development and testing it is highly recommended to turn
//! this on, as it sometimes also outputs more diagnostics.
//!
//! ### `dangerous_apis` (not recommended)
//!
//! There are some (unsafe) APIs that are not really necessary, but might in some
//! rare case be useful. Use only if you REALLY know what you do. For example there
//! are APIs that allow to change the resolution of a video frame while it is
//! allocated, which may lead to out-of-bounds memory access by safe code.

mod bindings;

pub mod blocking_update;
pub mod buffer_info;
pub mod enums;
pub mod find;
pub mod four_cc;
pub mod frame;
pub mod receiver;
pub mod resolution;
pub mod router;
pub mod sdk;
pub mod sender;
pub mod source;
pub mod subsampling;
pub mod tally;
pub mod timecode;
pub mod util;
