#![allow(non_upper_case_globals)]

use std::{ffi::CString, time::Duration};

use crate::{
    bindings::{self},
    enums::{NDIBandwidthMode, NDIColorFormat},
    frame::{AsFFIWritable, AudioFrame, FrameDataDropGuard, MetadataFrame, VideoFrame},
    structs::NDISourceLike,
};

#[derive(Debug, Clone)]
pub struct NDIReceiverBuilder<Source: NDISourceLike> {
    pub source: Option<Source>,
    pub name: Option<CString>,
    pub color_format: NDIColorFormat,
    pub bandwidth: NDIBandwidthMode,
    pub allow_fielded_video: bool,
}

impl<Source: NDISourceLike> Default for NDIReceiverBuilder<Source> {
    fn default() -> Self {
        Self {
            source: None,
            name: None,
            color_format: NDIColorFormat::default(),
            bandwidth: NDIBandwidthMode::default(),
            allow_fielded_video: true,
        }
    }
}

impl<Source: NDISourceLike> NDIReceiverBuilder<Source> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(CString::new(name).unwrap());
        self
    }

    pub fn color_format(mut self, color_format: NDIColorFormat) -> Self {
        self.color_format = color_format;
        self
    }

    pub fn bandwidth(mut self, bandwidth: NDIBandwidthMode) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn allow_fielded_video(mut self, allow: bool) -> Self {
        self.allow_fielded_video = allow;
        self
    }

    pub fn build(self) -> Option<NDIReceiver> {
        self.source.with_descriptor(|src_ptr| {
            let options = bindings::NDIlib_recv_create_v3_t {
                p_ndi_recv_name: self
                    .name
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(std::ptr::null()),

                source_to_connect_to: if src_ptr.is_null() {
                    bindings::NDIlib_source_t {
                        p_ndi_name: std::ptr::null(),
                        __bindgen_anon_1: bindings::NDIlib_source_t__bindgen_ty_1 {
                            p_url_address: std::ptr::null(),
                        },
                    }
                } else {
                    unsafe { *src_ptr }
                },
                color_format: self.color_format.to_ffi(),
                bandwidth: self.bandwidth.to_ffi(),
                allow_video_fields: self.allow_fielded_video,
            };

            let handle = unsafe { bindings::NDIlib_recv_create_v3(&options) };

            if handle.is_null() {
                return None;
            } else {
                Some(NDIReceiver { handle })
            }
        })
    }
}

pub struct NDIReceiver {
    handle: bindings::NDIlib_recv_instance_t,
}
unsafe impl Send for NDIReceiver {}
unsafe impl Sync for NDIReceiver {}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NDIRecvType {
    Video,
    Audio,
    Metadata,
    None,
    Unknown,
    StatusChange,
}

impl NDIReceiver {
    pub fn set_source(&self, source: impl NDISourceLike) {
        source.with_descriptor(|src_ptr| {
            unsafe { bindings::NDIlib_recv_connect(self.handle, src_ptr) };
        });
    }

    pub fn recv(
        &self,
        mut video: Option<&mut VideoFrame>,
        mut audio: Option<&mut AudioFrame>,
        mut meta: Option<&mut MetadataFrame>,
        timeout: Duration,
    ) -> NDIRecvType {
        let video_ptr = video.to_ffi_frame_ptr();
        if video_ptr.is_null() {
            video = None; // make sure NullPtr's are consistent with the option
        }

        let audio_ptr = audio.to_ffi_frame_ptr();
        if audio_ptr.is_null() {
            audio = None; // make sure NullPtr's are consistent with the option
        }

        let meta_ptr = meta.to_ffi_frame_ptr();
        if meta_ptr.is_null() {
            meta = None; // make sure NullPtr's are consistent with the option
        }

        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let recv_type = unsafe {
            bindings::NDIlib_recv_capture_v3(self.handle, video_ptr, audio_ptr, meta_ptr, timeout)
        };

        match recv_type {
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_video => {
                video
                    .expect(
                        "SDK indicated that a video frame was received, but there is no buffer it could have been written to",
                    )
                    .alloc = FrameDataDropGuard::Receiver(self.handle);

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                NDIRecvType::Video
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_audio => {
                audio
                    .expect(
                        "SDK indicated that an audio frame was received, but there is no buffer it could have been written to",
                    )
                    .alloc = FrameDataDropGuard::Receiver(self.handle);

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                NDIRecvType::Audio
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_metadata => {
                meta.expect(
                    "SDK indicated that a metadata frame was received, but there is no buffer it could have been written to",
                )
                .alloc = FrameDataDropGuard::Receiver(self.handle);

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                }

                NDIRecvType::Metadata
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_status_change => {
                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                NDIRecvType::StatusChange
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_none => {
                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                NDIRecvType::None
            }
            _ => NDIRecvType::Unknown,
        }
    }
}

impl Drop for NDIReceiver {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_recv_destroy(self.handle) };
    }
}
