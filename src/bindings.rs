#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::all)]
#![allow(unused_variables)]

#[cfg(not(any(docsrs, feature = "docsrs")))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(all(
    any(docsrs, feature = "docsrs"),
    target_os = "linux",
    target_arch = "x86_64",
))]
include!("./bindings/bindings.docsrs.x86_64-unknown-linux-gnu.rs.bin");
