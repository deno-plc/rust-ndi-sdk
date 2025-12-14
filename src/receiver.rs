//! NDI Receiver
//!
//! <https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-recv>

use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    ptr::NonNull,
    sync::Arc,
    time::Duration,
};

use static_assertions::assert_impl_all;

use crate::{
    bindings::{self},
    enums::{NDIBandwidthMode, NDIPreferredColorFormat, NDIRecvError},
    frame::{
        audio::AudioFrame,
        generic::{AsFFIReadable, AsFFIWritable},
        metadata::MetadataFrame,
        video::VideoFrame,
    },
    source::NDISourceLike,
    tally::Tally,
    util::duration_to_ms,
};

pub use crate::enums::NDIRecvType;

/// Builder for [NDIReceiver]
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

            if let Some(handle) = NonNull::new(handle) {
                Ok(NDIReceiver {
                    handle: Arc::new(RawReceiver { handle }),
                })
            } else {
                Err(NDIReceiverBuilderError::CreationFailed)
            }
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NDIReceiverBuilderError {
    CreationFailed,
}

#[derive(PartialEq, Eq)]
pub(crate) struct RawReceiver {
    handle: NonNull<bindings::NDIlib_recv_instance_type>,
}

impl RawReceiver {
    pub(crate) fn raw_ptr(&self) -> bindings::NDIlib_recv_instance_t {
        self.handle.as_ptr()
    }
}

impl Drop for RawReceiver {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_recv_destroy(self.raw_ptr()) };
    }
}

impl Debug for RawReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawReceiver")
            .field("raw_ptr", &self.raw_ptr())
            .finish()
    }
}

unsafe impl Send for RawReceiver {}
unsafe impl Sync for RawReceiver {}

/// A NDI receiver that can receive frames from a source.
///
/// Please note that the receiver handle (from the SDK) will not be dropped until all
/// frames that were received from it are dropped or have their buffers deallocated.[^note]
///
/// [^note]: The inner receiver is [Arc]ed because all frames received need to be dropped on the receiver handle and therefore need a valid reference to it
pub struct NDIReceiver {
    handle: Arc<RawReceiver>,
}

assert_impl_all!(NDIReceiver: Send, Sync);

impl NDIReceiver {
    /// Switches the receiver to the given source.
    pub fn set_source(&self, source: &impl NDISourceLike) {
        source.with_descriptor(|src_ptr| {
            unsafe { bindings::NDIlib_recv_connect(self.handle.raw_ptr(), src_ptr) };
        });
    }

    /// Tries to read into the given buffers and returns which of these has been written to.
    pub fn recv(
        &self,
        mut video: Option<&mut VideoFrame>,
        mut audio: Option<&mut AudioFrame>,
        mut meta: Option<&mut MetadataFrame>,
        timeout: Duration,
    ) -> Result<NDIRecvType, NDIRecvError> {
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

        let timeout: u32 = duration_to_ms(timeout);

        let recv_type = unsafe {
            bindings::NDIlib_recv_capture_v3(
                self.handle.raw_ptr(),
                video_ptr,
                audio_ptr,
                meta_ptr,
                timeout,
            )
        };

        match recv_type {
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_video => {
                video
                    .expect(
                        "[Fatal FFI Error] SDK indicated that a video frame was received, but there is no buffer it could have been written to",
                    )
                    .alloc.update_from_receiver(self.handle.clone());

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                Ok(NDIRecvType::Video)
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_audio => {
                audio
                    .expect(
                        "[Fatal FFI Error] SDK indicated that an audio frame was received, but there is no buffer it could have been written to",
                    )
                    .alloc.update_from_receiver(self.handle.clone());

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(meta) = meta {
                        meta.assert_unwritten();
                    }
                }

                Ok(NDIRecvType::Audio)
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_metadata => {
                meta.expect(
                    "[Fatal FFI Error] SDK indicated that a metadata frame was received, but there is no buffer it could have been written to",
                )
                .alloc.update_from_receiver(self.handle.clone());

                #[cfg(any(debug_assertions, feature = "strict_assertions"))]
                {
                    if let Some(video) = video {
                        video.assert_unwritten();
                    }
                    if let Some(audio) = audio {
                        audio.assert_unwritten();
                    }
                }

                Ok(NDIRecvType::Metadata)
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

                Ok(NDIRecvType::StatusChange)
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

                Ok(NDIRecvType::SourceChange)
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

                Ok(NDIRecvType::None)
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

                Err(NDIRecvError::UnknownType)
            }
            #[cfg(not(any(debug_assertions, feature = "strict_assertions")))]
            _ => NDIRecvType::Unknown,
        }
    }

    unsafe fn free_string(&self, ptr: *const std::os::raw::c_char) {
        if !ptr.is_null() {
            unsafe { bindings::NDIlib_recv_free_string(self.handle.raw_ptr(), ptr) };
        }
    }

    /// Sends a metadata frame over the current connection.
    pub fn send_metadata(&self, frame: &MetadataFrame) -> Result<(), SendMetadataError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendMetadataError::NotSendable(desc)
            }
        })?;

        let result = unsafe { bindings::NDIlib_recv_send_metadata(self.handle.raw_ptr(), ptr) };

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

        unsafe { bindings::NDIlib_recv_add_connection_metadata(self.handle.raw_ptr(), ptr) };

        Ok(())
    }

    /// Removes all connection metadata that was previously added.
    pub fn clear_connection_metadata(&self) {
        unsafe { bindings::NDIlib_recv_clear_connection_metadata(self.handle.raw_ptr()) };
    }

    /// Sets the tally status for the receiver. This will be merged from all receivers on the same
    /// source.
    pub fn set_tally(&self, tally: Tally) {
        let tally = tally.to_ffi();

        unsafe { bindings::NDIlib_recv_set_tally(self.handle.raw_ptr(), &tally) };
    }

    /// Returns the current number of connections to the receiver.
    pub fn get_num_connections(&self) -> usize {
        let num_connections =
            unsafe { bindings::NDIlib_recv_get_no_connections(self.handle.raw_ptr()) };
        num_connections
            .try_into()
            .expect("[Fatal FFI Error] NDI SDK returned a invalid number of connections")
    }

    /// Get the web control URL for the receiver.
    pub fn get_web_control(&self) -> Option<NDIWebControlInfo<'_>> {
        let ptr = unsafe { bindings::NDIlib_recv_get_web_control(self.handle.raw_ptr()) };

        if ptr.is_null() {
            None?;
        }

        let str = unsafe { CStr::from_ptr(ptr) };
        Some(NDIWebControlInfo {
            url: str,
            recv: self,
        })
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
