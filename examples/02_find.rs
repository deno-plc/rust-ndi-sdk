use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

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

    let mut source_list = HashSet::new();

    let start = Instant::now();

    let duration = Duration::from_secs(2);

    while start.elapsed() < duration {
        let sources = finder.get_source_iter().unwrap();

        for source in sources {
            let source = source.to_owned();
            if source_list.insert(source.clone()) {
                println!("Found source: {:?}", source);
            }
        }

        let elapsed = start.elapsed();

        if duration > elapsed {
            finder.blocking_wait_for_change(duration - elapsed);
        }
    }

    if source_list.is_empty() {
        println!("No sources found");
    }
}
