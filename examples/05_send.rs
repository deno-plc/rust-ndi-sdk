use std::time::Instant;

use ndi_sdk_sys::{
    four_cc::FourCCVideo, frame::video::VideoFrame, sdk, sender::NDISenderBuilder,
    structs::resolution::Resolution,
};

fn main() {
    let v = sdk::version();

    println!("{}", v.unwrap_or("NDI SDK version unavailable"));

    sdk::initialize().unwrap();

    println!("NDI initialized successfully");

    let mut frame = VideoFrame::new();

    let src = NDISenderBuilder::new()
        .name("My Test Source")
        .clock_video(true)
        .build()
        .unwrap();

    println!(
        "Discoverable as \"{}\"",
        src.get_source().name().to_str().unwrap()
    );

    frame.alloc(Resolution::new(1920, 1080), FourCCVideo::RGBX);

    let start = Instant::now();

    loop {
        let (buf, info) = frame.video_data_mut().unwrap();

        let b = (f32::sin(start.elapsed().as_secs_f32()) * 256.0) as u8 + 0xff / 2;

        for x in 0..info.resolution.x {
            let r = (x * 0xff / info.resolution.x) as u8;
            for y in 0..info.resolution.y {
                let offset = (y * info.resolution.x + x) * 4;

                let a = 0xff;
                let g = 0xff - r;

                buf[offset + 0] = r;
                buf[offset + 1] = g;
                buf[offset + 2] = b;
                buf[offset + 3] = a;
            }
        }

        src.send_video_sync(&frame).unwrap();
    }
}
