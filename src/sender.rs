use std::{
    ffi::CString,
    fmt::Debug,
    ptr::NonNull,
    sync::{Arc, Mutex},
    time::Duration,
};

use static_assertions::assert_impl_all;

use crate::{
    bindings,
    frame::{
        audio::AudioFrame,
        generic::{AsFFIReadable, AsFFIWritable},
        metadata::MetadataFrame,
        video::VideoFrame,
    },
    source::{NDISourceLike, NDISourceRef},
    structs::{BlockingUpdate, tally::Tally},
};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct NDISenderBuilder {
    pub name: Option<CString>,
    pub groups: Option<CString>,
    pub clock_video: bool,
    pub clock_audio: bool,
}

impl Default for NDISenderBuilder {
    fn default() -> Self {
        Self {
            name: None,
            groups: None,
            clock_video: false,
            clock_audio: false,
        }
    }
}

impl NDISenderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name for the sender.
    ///
    /// The total length of an NDI source name should be limited to 253 characters. The following characters
    /// are considered invalid: \ / : * ? " < > |. If any of these characters are found in the name, they will
    /// be replaced with a space. These characters are reserved according to Windows file system naming conventions
    pub fn name(mut self, name: &str) -> Result<Self, SenderNameError> {
        let name = CString::new(name).map_err(|e| SenderNameError::NulError(e))?;
        if name.count_bytes() >= 253 {
            Err(SenderNameError::TooLong)?;
        }
        self.name = Some(name);
        Ok(self)
    }

    /// Sets the groups for the sender.
    ///
    /// This parameter represents the groups that this NDI sender should place itself into. Groups are sets of
    /// NDI sources. Any source can be part of any number of groups, and groups are comma-separated.
    /// For instance, "cameras,studio 1,10am show" would place a source in the three groups named.
    /// On the finding side, you can specify which groups to look for and look in multiple groups.
    /// If the group is not set then the system default groups will be used.
    /// https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#parameters
    pub fn groups(mut self, groups: &str) -> Self {
        self.groups = Some(CString::new(groups).unwrap());
        self
    }

    /// When enabled the SDK will limit the frame rate for video frames.
    /// The send function will block until the next frame is ready to be sent.
    /// https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#parameters
    pub fn clock_video(mut self, clock_video: bool) -> Self {
        self.clock_video = clock_video;
        self
    }

    /// When enabled the SDK will limit the frame rate for audio frames.
    /// The send function will block until the next frame is ready to be sent.
    /// https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#parameters
    pub fn clock_audio(mut self, clock_audio: bool) -> Self {
        self.clock_audio = clock_audio;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SenderNameError {
    NulError(std::ffi::NulError),
    TooLong,
}

impl NDISenderBuilder {
    pub fn build(self) -> Result<NDISender, NDISenderBuilderError> {
        let options = bindings::NDIlib_send_create_t {
            p_ndi_name: self.name.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            p_groups: self
                .groups
                .as_ref()
                .map_or(std::ptr::null(), |s| s.as_ptr()),
            clock_video: self.clock_video,
            clock_audio: self.clock_audio,
        };

        let handle = unsafe { bindings::NDIlib_send_create(&options) };

        if let Some(handle) = NonNull::new(handle) {
            Ok(NDISender {
                handle: Arc::new(RawSender { handle }),
                in_transmission: Mutex::new(None),
            })
        } else {
            Err(NDISenderBuilderError::CreationFailed)
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NDISenderBuilderError {
    CreationFailed,
}

#[derive(PartialEq, Eq)]
pub(crate) struct RawSender {
    handle: NonNull<bindings::NDIlib_send_instance_type>,
}
impl RawSender {
    pub(crate) fn raw_ptr(&self) -> bindings::NDIlib_send_instance_t {
        self.handle.as_ptr()
    }
}

impl Drop for RawSender {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_send_destroy(self.raw_ptr()) };
    }
}

impl Debug for RawSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawSender")
            .field("raw_ptr", &self.raw_ptr())
            .finish()
    }
}

unsafe impl Send for RawSender {}
unsafe impl Sync for RawSender {}

/// A NDI sender that can send frames to a receiver.
///
/// Please note that the sender handle will not be dropped until all metadata frames
/// that were received from it are dropped or have their buffers deallocated. If you
/// dynamically create and destroy senders take into consideration that metadata frames
/// may prevent the sender resources from being released.
pub struct NDISender {
    handle: Arc<RawSender>,
    in_transmission: Mutex<Option<Arc<VideoFrame>>>,
}

assert_impl_all!(
    NDISender: Send, Sync,
);

impl NDISender {
    /// Sends a video frame
    /// This will block until the frame is sent.
    pub fn send_video_sync(&self, frame: &VideoFrame) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_video_v2(self.handle.raw_ptr(), ptr);
            self.write_in_transmission(None);
        }

        Ok(())
    }

    /// This updates the Arc reference for the frame that is held in the background of async transmissions
    ///
    /// SAFETY:
    /// This function may drop the frame of a previous transmission through the Arc. If this is called to
    /// early, it may lead to a use-after-free error and frame glitches.
    ///
    /// https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#asynchronous-sending
    unsafe fn write_in_transmission(&self, frame: Option<Arc<VideoFrame>>) {
        match self.in_transmission.lock() {
            Ok(mut guard) => {
                *guard = frame;
            }
            Err(mut e) => {
                **e.get_mut() = frame;
                self.in_transmission.clear_poison();
            }
        }
    }

    /// Sends a video frame asynchronously.
    ///
    /// > https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#asynchronous-sending
    /// >
    /// > This function will return immediately and will perform all required operations (including color conversion,
    /// > compression, and network transmission) asynchronously with the call.
    /// > Because NDI takes full advantage of asynchronous OS behavior when available, this will normally result in
    /// > improved performance (as compared to creating your own thread and submitting frames asynchronously with rendering).
    ///
    /// Blocking: This function does not block by default, but it will block in the following cases:
    /// - The previous frame is still in transmission
    /// - The sender uses [NDISenderBuilder::clock_video]
    ///
    /// This will hold a strong reference to the frame until it is guaranteed to be safely mutated again. This is:
    /// - After a call to `flush_async_video`
    /// - After the next call to `send_video_async`
    /// - After a call to `send_video_sync`
    /// - When the `NDISender` is dropped (this is not affected by delayed dropping of the sender handle)
    pub fn send_video_async(&self, frame: Arc<VideoFrame>) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_video_async_v2(self.handle.raw_ptr(), ptr);
            self.write_in_transmission(Some(frame));
        }

        Ok(())
    }

    /// Blocks until the current async video transmission is finished.
    pub fn flush_async_video(&self) {
        unsafe {
            bindings::NDIlib_send_send_video_async_v2(self.handle.raw_ptr(), std::ptr::null_mut());
            self.write_in_transmission(None);
        }
    }

    pub fn send_audio(&self, frame: &AudioFrame) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_audio_v3(self.handle.raw_ptr(), ptr);
        }

        Ok(())
    }

    pub fn send_metadata(&self, frame: &MetadataFrame) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_metadata(self.handle.raw_ptr(), ptr);
        }

        Ok(())
    }

    /// Receives a metadata frame, works exactly like [super::receiver::NDIReceiver::recv]
    pub fn recv_metadata(
        &self,
        meta: &mut MetadataFrame,
        timeout: Duration,
    ) -> Result<Option<()>, ()> {
        let ptr = meta.to_ffi_recv_frame_ptr();

        if ptr.is_null() {
            return Err(());
        }

        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let recv_type =
            unsafe { bindings::NDIlib_send_capture(self.handle.raw_ptr(), ptr, timeout) };

        match recv_type {
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_video
            | bindings::NDIlib_frame_type_e_NDIlib_frame_type_audio => {
                panic!("[Fatal FFI Error] Invalid enum discriminant");
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_metadata => {
                meta.alloc.from_sender(self.handle.clone());
                Ok(Some(()))
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_none => Ok(None),
            discriminant => {
                eprintln!("NDI SDK returned an unknown frame type: {:?}", discriminant);

                meta.assert_unwritten();

                Err(())
            }
        }
    }

    /// Returns the current tally state.
    pub fn get_tally(&self) -> Tally {
        let mut tally = bindings::NDIlib_tally_t {
            on_program: false,
            on_preview: false,
        };

        unsafe { bindings::NDIlib_send_get_tally(self.handle.raw_ptr(), &mut tally, 0) };

        Tally::from_ffi(&tally)
    }

    /// Blocks until the tally state changes or the timeout is reached.
    pub fn get_tally_update(&self, timeout: Duration) -> BlockingUpdate<Tally> {
        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let mut tally = bindings::NDIlib_tally_t {
            on_program: false,
            on_preview: false,
        };

        let changed = unsafe {
            bindings::NDIlib_send_get_tally(self.handle.raw_ptr(), &mut tally, timeout) == true
        };

        BlockingUpdate::new(Tally::from_ffi(&tally), changed)
    }

    /// Blocks until the number of connections changes or the timeout is reached.
    pub fn get_num_connections_update(&self, timeout: Duration) -> usize {
        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let no_conns =
            unsafe { bindings::NDIlib_send_get_no_connections(self.handle.raw_ptr(), timeout) };

        no_conns
            .try_into()
            .expect("[Fatal FFI Error] NDI SDK returned an invalid number of connections")
    }

    /// Sets the failover source, which is used when the receiver cannot receive frames from this source anymore.
    /// https://docs.ndi.video/all/developing-with-ndi/sdk/ndi-send#failsafe
    pub fn set_failover(&self, failover_src: impl NDISourceLike) {
        failover_src.with_descriptor(|src_ptr| {
            unsafe { bindings::NDIlib_send_set_failover(self.handle.raw_ptr(), src_ptr) };
        });
    }

    /// Get this sources description (includes the name)
    pub fn get_source<'a>(&'a self) -> NDISourceRef<'a> {
        let source = unsafe { bindings::NDIlib_send_get_source_name(self.handle.raw_ptr()) };

        unsafe {
            NDISourceRef::from(
                source
                    .as_ref()
                    .expect("[Fatal FFI Error] NDI SDK returned nullptr for source descriptor"),
            )
        }
    }

    /// Adds connection metadata which will be sent to every connection in the future.
    pub fn add_connection_metadata(&self, meta: &MetadataFrame) -> Result<(), SendFrameError> {
        let ptr = meta.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_add_connection_metadata(self.handle.raw_ptr(), ptr);
        }

        Ok(())
    }

    /// Removes all connection metadata that was previously added.
    pub fn clear_connection_metadata(&self) {
        unsafe { bindings::NDIlib_send_clear_connection_metadata(self.handle.raw_ptr()) };
    }
}

impl Drop for NDISender {
    fn drop(&mut self) {
        // This is required because dropping the sender will also (potentially) drop the in-transmission frame,
        // but the sender handle may outlive this if it has to wait for a metadata frame to be dropped.
        self.flush_async_video();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SendFrameError {
    NotSendable(&'static str),
}
