use std::{env, time::Duration};

use ndi_sdk_sys::{
    enums::NDIColorFormat,
    four_cc::FourCCVideo,
    frame::{metadata::MetadataFrame, video::VideoFrame},
    receiver::{NDIReceiverBuilder, NDIRecvType},
    structs::NDISource,
    *,
};

fn main() {
    let source_name = env::args()
        .collect::<Vec<_>>()
        .get(1)
        .cloned()
        .expect("Please provide a source name");

    let v = sdk::version();

    println!("{}", v.unwrap_or("NDI SDK version unavailable"));

    sdk::initialize().unwrap();

    println!("NDI initialized successfully");

    let receiver = NDIReceiverBuilder::new()
        .source(NDISource::from_name(&source_name))
        .allow_fielded_video(false)
        .color_format(
            NDIColorFormat::from_four_cc(Some(FourCCVideo::RGBA), Some(FourCCVideo::RGBX)).unwrap(),
        )
        .build()
        .unwrap();

    let mut video = VideoFrame::new();
    let mut metadata = MetadataFrame::new();

    loop {
        match receiver.recv(
            Some(&mut video),
            None,
            Some(&mut metadata),
            Duration::from_secs(1),
        ) {
            NDIRecvType::Video => {
                println!("Received video frame {:?}", video);
                let data = video.video_data();
                if let Some(data) = data {
                    println!("Video data: {:?} ...", &data[0..16]);
                }
                video = VideoFrame::new();
            }
            NDIRecvType::Metadata => {
                println!("Received metadata frame");
                metadata = MetadataFrame::new();
            }
            NDIRecvType::StatusChange => {
                println!("Status change");
            }
            NDIRecvType::None => {}
            t => {
                println!("Received {:?}", t);
            }
        }
    }
}
