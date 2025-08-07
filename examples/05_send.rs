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

    let src = NDISenderBuilder::new()
        .name("My Test Source")
        .unwrap()
        .clock_video(true)
        .build()
        .unwrap();

    println!(
        "Discoverable as \"{}\"",
        src.get_source().name().to_str().unwrap()
    );

    let mut frame = VideoFrame::new();
    frame.set_resolution(Resolution::new(1920, 1080)).unwrap();
    frame.set_four_cc(FourCCVideo::RGBX).unwrap();

    frame.try_alloc().unwrap();

    let start = Instant::now();

    loop {
        let (buf, info) = frame.video_data_mut().unwrap();

        let b = (f32::sin(start.elapsed().as_secs_f32()) * 128.0 + 128.0) as u8;

        for x in 0..info.resolution.x {
            let r = (x * 0xff / info.resolution.x) as u8;
            for y in 0..info.resolution.y {
                let g = (y * 0xff / info.resolution.y) as u8;
                let a = 0xff;

                let offset = (y * info.resolution.x + x) * 4;
                buf[offset + 0] = r;
                buf[offset + 1] = g;
                buf[offset + 2] = b;
                buf[offset + 3] = a;
            }
        }

        src.send_video_sync(&frame).unwrap();
    }
}
