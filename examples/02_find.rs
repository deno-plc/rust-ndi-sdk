use std::time::Duration;

use ndi_sdk_sys::{find::NDISourceFinderBuilder, *};

fn main() {
    let v = sdk::version();

    println!("{}", v.unwrap_or("NDI SDK version unavailable"));

    sdk::initialize().unwrap();

    println!("NDI initialized successfully");

    let mut finder = NDISourceFinderBuilder::default()
        .show_local_sources(true)
        .build()
        .unwrap();

    let mut source_list = Vec::new();

    while source_list.is_empty() {
        let sources = finder.get_source_iter().unwrap();

        source_list.clear();

        for source in sources {
            let source = source.to_owned();
            source_list.push(source);
        }

        if source_list.is_empty() {
            finder.blocking_wait_for_change(Duration::from_secs(1));
        }
    }

    for source in &source_list {
        println!("Found source: {}", source.name());
    }
}
