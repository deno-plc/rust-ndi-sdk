# rust-ndi-sdk (`ndi-sdk-sys`)

[![Crate](https://img.shields.io/crates/v/ndi-sdk-sys.svg)](https://crates.io/crates/ndi-sdk-sys)
[![Documentation](https://docs.rs/ndi-sdk-sys/badge.svg)](https://docs.rs/ndi-sdk-sys)

Safe and ergonomic Rust bindings for the [NDI (ndi.video)](https://ndi.video/)
SDK, enabling high-performance video, audio, and metadata transmission over IP
networks.

Unlike other bindings this never exposes any raw pointers and uses correct Rust
types for everything (timeouts are handled as `std::time::Duration`, not
`time_in_ms: u32`)

Despite its name this is not affiliated with `ndi`, `ndi-sdk`, `ndi-sys` crates

## Features

- Safe wrapper around the NDI SDK
- Currently supports:
  - Finder API for discovering NDI sources on the network
  - Router API for routing NDI streams
  - Sender API for transmitting NDI streams
  - Receiver API for receiving NDI streams
- Not supported yet:
  - Dynamic loading of NDI SDK
  - PTZ Control
  - FrameSync
  - Receiver advertisement
  - Audio frame access
  - video frame metadata write

## Version compatibility

This crate is currently tested against SDK version 6.2.0.3

The raw bindings are generated from the header files in the SDK installation
directory, this way generated code will always match the version on the machine
it is compiled with and linked against.

## Platform Support

This crates `build.rs` is currently lacking support for platforms other than
Windows x64, because I had no time for it so far. If you are working on another
platform, feel free to add support for it. (It shouldn't be that complicated,
windows support is less than 30 lines of nearly-boilerplate code)

## Installation

1. Make sure to have the [NDI SDK](https://ndi.video/for-developers/#ndi-sdk)
   installed
2. Make sure to have Clang/LLVM installed.
   [see rust-bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)
3. Add `ndi-sdk-sys` to your cargo dependencies

Check out the examples at
https://github.com/deno-plc/rust-ndi-sdk/tree/main/examples

## Cargo features

### `strict_assertions`

This crate uses a lot of unsafe ffi bindings and therefore a lot of assertions.

You can turn on this feature to ensure debug assertions are in place even for
release builds. During development and testing it is highly recommended to turn
this on, as it sometimes also outputs more diagnostics.

### `dangerous_apis` (not recommended)

There are some (unsafe) APIs that are not really necessary, but might in some
rare case be useful. Use only if you REALLY know what you do. For example there
are APIs that allow to change the resolution of a video frame while it is
allocated, which may lead to out-of-bounds memory access by safe code.

## Safety

This crate provides safe abstractions of the NDI SDK. The public API is designed
to be 100% safe to use (except unsafe APIs of course). If you encounter
restrictive lifetimes or &mut borrows, do not try to circumvent them, most
likely it just represents the safety requirements of the C SDK.

As expected form a C library, safety requirements are not always documented
in-depth by the NDI SDK. One common assumption this crate uses it that it is
safe to drop arguments that are passed as pointers (and strings inside them)
immediately after the call they were passed to has returned (unless defined
otherwise by the documentation).

## Contributing

Contributions are welcome! Here's how you can help:

1. Add support for other platforms in `build.rs`
2. Report bugs and find missing APIs
3. Implement these missing APIs
4. Improve documentation and examples

## License

Copyright (C) 2025 Hans Schallmoser

Licensed under GPL-v3.

A lot of the doc comments is adapted from the
[NDI Docs](https://docs.ndi.video/all/developing-with-ndi/sdk)

Note: The NDI SDK is subject to its own
[EULA](https://ndi.video/for-developers/#ndi-sdk).

## Disclaimer

This project is not affiliated with NDI® or NewTek/Vizrt
