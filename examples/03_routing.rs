use std::{thread::sleep, time::Duration};

use ndi_sdk_sys::{find::NDISourceFinderBuilder, router::NDIRouterBuilder, *};

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
        println!("Found source: {:?}", source);
    }

    let mut router = NDIRouterBuilder::new("Test Router").build().unwrap();

    let mut i = 0usize;

    loop {
        let source = source_list.get(i).unwrap_or_else(|| {
            i = 0;
            source_list.get(0).unwrap()
        });
        println!("Switching to source: {}", source.name());
        router.switch(source).unwrap();
        i += 1;
        sleep(Duration::from_secs(5));
    }
}
