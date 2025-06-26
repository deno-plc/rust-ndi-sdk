use std::{ffi::CString, time::Duration};

use crate::{
    bindings,
    frame::{
        audio::AudioFrame,
        drop_guard::FrameDataDropGuard,
        generic::{AsFFIReadable, AsFFIWritable},
        metadata::MetadataFrame,
        video::VideoFrame,
    },
    source::{NDISourceLike, NDISourceRef},
    structs::{BlockingUpdate, Tally},
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

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(CString::new(name).unwrap());
        self
    }

    pub fn groups(mut self, groups: &str) -> Self {
        self.groups = Some(CString::new(groups).unwrap());
        self
    }

    pub fn clock_video(mut self, clock_video: bool) -> Self {
        self.clock_video = clock_video;
        self
    }

    pub fn clock_audio(mut self, clock_audio: bool) -> Self {
        self.clock_audio = clock_audio;
        self
    }
}

impl NDISenderBuilder {
    pub fn build(self) -> Result<NDISender, ()> {
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

        if handle.is_null() {
            Err(())
        } else {
            Ok(NDISender { handle })
        }
    }
}

pub struct NDISender {
    handle: bindings::NDIlib_send_instance_t,
}
unsafe impl Send for NDISender {}
unsafe impl Sync for NDISender {}

impl NDISender {
    pub fn send_video_sync(&self, frame: &VideoFrame) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_video_v2(self.handle, ptr);
        }

        Ok(())
    }

    pub fn send_audio(&self, frame: &AudioFrame) -> Result<(), SendFrameError> {
        let ptr = frame.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_send_audio_v3(self.handle, ptr);
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
            bindings::NDIlib_send_send_metadata(self.handle, ptr);
        }

        Ok(())
    }

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

        let recv_type = unsafe { bindings::NDIlib_send_capture(self.handle, ptr, timeout) };

        match recv_type {
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_video
            | bindings::NDIlib_frame_type_e_NDIlib_frame_type_audio => {
                panic!("[Fatal FFI Error] Invalid enum discriminant");
            }
            bindings::NDIlib_frame_type_e_NDIlib_frame_type_metadata => {
                meta.alloc = FrameDataDropGuard::Sender(self.handle);
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

    pub fn get_tally(&self) -> Tally {
        let mut tally = bindings::NDIlib_tally_t {
            on_program: false,
            on_preview: false,
        };

        unsafe { bindings::NDIlib_send_get_tally(self.handle, &mut tally, 0) };

        Tally::from_ffi(&tally)
    }

    /// Blocks until the tally state changes or the timeout is reached.
    pub fn get_tally_update(&self, timeout: Duration) -> BlockingUpdate<Tally> {
        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let mut tally = bindings::NDIlib_tally_t {
            on_program: false,
            on_preview: false,
        };

        let changed = unsafe { bindings::NDIlib_send_get_tally(self.handle, &mut tally, timeout) };

        BlockingUpdate::new(Tally::from_ffi(&tally), changed)
    }

    pub fn get_num_connections_update(&self, timeout: Duration) -> usize {
        let timeout: u32 = timeout.as_millis().try_into().unwrap_or(u32::MAX);

        let no_conns = unsafe { bindings::NDIlib_send_get_no_connections(self.handle, timeout) };

        no_conns
            .try_into()
            .expect("[Fatal FFI Error] NDI SDK returned an invalid number of connections")
    }

    pub fn set_failover(&self, failover_src: impl NDISourceLike) {
        failover_src.with_descriptor(|src_ptr| {
            unsafe { bindings::NDIlib_send_set_failover(self.handle, src_ptr) };
        });
    }

    pub fn get_source<'a>(&'a self) -> NDISourceRef<'a> {
        let source = unsafe { bindings::NDIlib_send_get_source_name(self.handle) };

        unsafe {
            NDISourceRef::from(
                source
                    .as_ref()
                    .expect("[Fatal FFI Error] NDI SDK returned nullptr for source descriptor"),
            )
        }
    }

    pub fn clear_connection_metadata(&self) {
        unsafe { bindings::NDIlib_send_clear_connection_metadata(self.handle) };
    }

    pub fn add_connection_metadata(&self, meta: &MetadataFrame) -> Result<(), SendFrameError> {
        let ptr = meta.to_ffi_send_frame_ptr().map_err(|err| match err {
            crate::frame::generic::FFIReadablePtrError::NotReadable(desc) => {
                SendFrameError::NotSendable(desc)
            }
        })?;

        unsafe {
            bindings::NDIlib_send_add_connection_metadata(self.handle, ptr);
        }

        Ok(())
    }
}

pub enum SendFrameError {
    NotSendable(&'static str),
}

impl Drop for NDISender {
    fn drop(&mut self) {
        unsafe { bindings::NDIlib_send_destroy(self.handle) };
    }
}
