use std::{
    ffi::{CStr, CString},
    time::Duration,
};

use crate::{
    bindings::{self},
    enums::{NDIBandwidthMode, NDIPreferredColorFormat},
    frame::{
        audio::AudioFrame,
        drop_guard::FrameDataDropGuard,
        generic::{AsFFIReadable, AsFFIWritable},
        metadata::MetadataFrame,
        video::VideoFrame,
    },
    source::NDISourceLike,
    structs::tally::Tally,
};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NDIReceiverBuilder<Source: NDISourceLike> {
    pub source: Option<Source>,
    pub name: Option<CString>,
    pub color_format: NDIPreferredColorFormat,
    pub bandwidth: NDIBandwidthMode,
    pub allow_fielded_video: bool,
}

impl<Source: NDISourceLike> Default for NDIReceiverBuilder<Source> {
    fn default() -> Self {
        Self {
            source: None,
            name: None,
            color_format: NDIPreferredColorFormat::default(),
            bandwidth: NDIBandwidthMode::default(),
            allow_fielded_video: false,
        }
    }
}

impl<Source: NDISourceLike> NDIReceiverBuilder<Source> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the source to connect to. Can be left out to select the source later.
    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the receiver name. This will be used in future versions of NDI.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(CString::new(name).unwrap());
        self
    }

    /// Sets the preferred color format for the receiver.
    /// Unless you need a specific color format, always use [NDIPreferredColorFormat::Fastest]
    pub fn color_format(mut self, color_format: NDIPreferredColorFormat) -> Self {
        self.color_format = color_format;
        self
    }

    /// Sets the bandwidth mode for the receiver.
    /// Can be used to use reduced bandwidth for multi-views or metadata-only
    pub fn bandwidth(mut self, bandwidth: NDIBandwidthMode) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    /// Sets whether the receiver should allow fielded video.
    /// Default is false, only progressive video will be received.
    pub fn allow_fielded_video(mut self, allow: bool) -> Self {
        self.allow_fielded_video = allow;
        self
    }

    pub fn build(self) -> Result<NDIReceiver, NDIReceiverBuilderError> {
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
                Err(NDIReceiverBuilderError::CreationFailed)
            } else {
                Ok(NDIReceiver { handle })
            }
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NDIReceiverBuilderError {
    CreationFailed,
}

pub struct NDIReceiver {
    handle: bindings::NDIlib_recv_instance_t,
}
unsafe impl Send for NDIReceiver {}
unsafe impl Sync for NDIReceiver {}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NDIRecvType {
    /// A video frame was received
    Video,
    /// An audio frame was received
    Audio,
    /// A metadata frame was received
    Metadata,
    /// No frame was received, most likely because the timeout was reached.
    None,
    /// The SDK returned a frame type that is not recognized.
    Unknown,
    /// No frame was received, but the status of the connection changed.
    /// Things like the web control URL could have changed
    StatusChange,
    /// The source the receiver is connected to has changed
    SourceChange,
}

impl NDIReceiver {
    /// Switches the receiver to the given source.
    pub fn set_source(&self, source: impl NDISourceLike) {
        source.with_descriptor(|src_ptr| {
            unsafe { bindings::NDIlib_recv_connect(self.handle, src_ptr) };
        });
    }

    /// Tries to read into the given buffers and returns which of these has been written to.
    pub fn recv(
        &self,
        mut video: Option<&mut VideoFrame>,
        mut audio: Option<&mut AudioFrame>,
        mut meta: Option<&mut MetadataFrame>,
        timeout: Duration,
    ) -> NDIRecvType {
        let video_ptr = video.to_ffi_recv_frame_ptr();
        if video_ptr.is_null() {
            video = None; // make sure NullPtr's are consistent with the option
        }

        let audio_ptr = audio.to_ffi_recv_frame_ptr();
        if audio_ptr.is_null() {
            audio = None; // make sure NullPtr's are consistent with the option
        }

        let meta_ptr = meta.to_ffi_recv_frame_ptr();
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
                        "[Fatal FFI Error] SDK indicated that a video frame was received, but there is no buffer it could have been written to",
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
                        "[Fatal FFI Error] SDK indicated that an audio frame was received, but there is no buffer it could have been written to",
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
                    "[Fatal FFI Error] SDK indicated that a metadata frame was received, but there is no buffer it could have been written to",
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
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_source_change => {
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

                NDIRecvType::SourceChange
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
            #[cfg(any(debug_assertions, feature = "strict_assertions"))]
            discriminant => {
                eprintln!("NDI SDK returned an unknown frame type: {:?}", discriminant);

                if let Some(video) = video {
                    video.assert_unwritten();
                }
                if let Some(audio) = audio {
                    audio.assert_unwritten();
                }
                if let Some(meta) = meta {
                    meta.assert_unwritten();
                }

                NDIRecvType::Unknown
            }
            #[cfg(not(any(debug_assertions, feature = "strict_assertions")))]
            _ => NDIRecvType::Unknown,
        }
    }

    unsafe fn free_string(&self, ptr: *const std::os::raw::c_char) {
        if !ptr.is_null() {
            unsafe { bindings::NDIlib_recv_free_string(self.handle, ptr) };
        }
    }

    /// Sends a metadata frame over the current connection.
    pub fn send_metadata(&self, frame: &MetadataFrame) -> Result<(), SendMetadataError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendMetadataError::NotSendable(desc)
            }
        })?;

        let result = unsafe { bindings::NDIlib_recv_send_metadata(self.handle, ptr) };

        if result {
            Ok(())
        } else {
            Err(SendMetadataError::NotConnected)
        }
    }

    /// Adds connection metadata which will be sent to every connection in the future.
    pub fn add_connection_metadata(&self, frame: &MetadataFrame) -> Result<(), SendMetadataError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendMetadataError::NotSendable(desc)
            }
        })?;

        unsafe { bindings::NDIlib_recv_add_connection_metadata(self.handle, ptr) };

        Ok(())
    }

    /// Removes all connection metadata that was previously added.
    pub fn clear_connection_metadata(&self) {
        unsafe { bindings::NDIlib_recv_clear_connection_metadata(self.handle) };
    }

    /// Sets the tally status for the receiver. This will be merged from all receivers on the same
    /// source.
    pub fn set_tally(&self, tally: Tally) {
        let tally = tally.to_ffi();

        unsafe { bindings::NDIlib_recv_set_tally(self.handle, &tally) };
    }

    /// Returns the current number of connections to the receiver.
    pub fn get_num_connections(&self) -> usize {
        let num_connections = unsafe { bindings::NDIlib_recv_get_no_connections(self.handle) };
        num_connections
            .try_into()
            .expect("[Fatal FFI Error] NDI SDK returned a invalid number of connections")
    }

    /// Get the web control URL for the receiver.
    pub fn get_web_control(&self) -> Option<NDIWebControlInfo> {
        let ptr = unsafe { bindings::NDIlib_recv_get_web_control(self.handle) };

        if ptr.is_null() {
            return None;
        }

        let str = unsafe { CStr::from_ptr(ptr) };
        Some(NDIWebControlInfo {
            url: str,
            recv: self,
        })
    }
}

impl Drop for NDIReceiver {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_recv_destroy(self.handle) };
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SendMetadataError {
    /// The metadata could not be sent because it is not readable
    NotSendable(&'static str),
    /// The metadata could not be sent because the receiver is not connected
    NotConnected,
}

pub struct NDIWebControlInfo<'a> {
    url: &'a CStr,
    recv: &'a NDIReceiver,
}

impl<'a> Drop for NDIWebControlInfo<'a> {
    fn drop(&mut self) {
        unsafe { self.recv.free_string(self.url.as_ptr()) };
    }
}

impl<'a> NDIWebControlInfo<'a> {
    pub fn as_cstr(&self) -> &'a CStr {
        self.url
    }
}
