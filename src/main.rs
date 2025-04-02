use core::panic;
use std::{ffi::CStr, thread::sleep, time::Duration};

use rust_ndi_sdk::{find::NDISourceFinderBuilder, router::NDIRouterBuilder, *};

fn main() {
    let v = unsafe { CStr::from_ptr(bindings::NDIlib_version()) };

    println!("{}", v.to_str().unwrap());

    let cpu_supported = unsafe { bindings::NDIlib_is_supported_CPU() };

    if !cpu_supported {
        panic!("CPU not supported");
    }

    let status = unsafe { bindings::NDIlib_initialize() };

    if !status {
        panic!("Failed to initialize NDI");
    }

    println!("NDI initialized successfully");

    let mut finder = NDISourceFinderBuilder::default()
        .show_local_sources(true)
        .build()
        .unwrap();

    let mut source_list = Vec::new();

    while source_list.is_empty() {
        let sources = finder.get_sources().unwrap();

        source_list.clear();

        for source in sources {
            let source = source.to_owned();
            // println!("Found source: {}", source.name());
            source_list.push(source);
        }

        if source_list.is_empty() {
            finder.blocking_wait_for_change(Duration::from_secs(1));
        }
    }

    for source in &source_list {
        println!("Found source: {}", source.name());
    }

    let mut router = NDIRouterBuilder::new("Test Router").build().unwrap();

    let mut i = 0usize;

    // loop {
    let source = source_list.get(i).unwrap_or_else(|| {
        i = 0;
        source_list.get(0).unwrap()
    });
    println!("Switching to source: {}", source.name());
    router.switch(source).unwrap();
    // i += 1;
    sleep(Duration::from_secs(5));
    // }
}
