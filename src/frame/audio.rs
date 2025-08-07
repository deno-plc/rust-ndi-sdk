use std::sync::Arc;

use super::{NDIFrame, RawBufferManagement, RawFrame, drop_guard::FrameDataDropGuard};

pub(crate) use crate::bindings::NDIlib_audio_frame_v3_t as NDIRawAudioFrame;
use crate::{bindings, four_cc::FourCCAudio, receiver::RawReceiver, sender::RawSender};

impl RawBufferManagement for NDIRawAudioFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: &Arc<RawReceiver>) {
        unsafe { bindings::NDIlib_recv_free_audio_v3(recv.raw_ptr(), self) }
    }

    #[inline]
    unsafe fn drop_with_sender(&mut self, _sender: &Arc<RawSender>) {
        panic!(
            "NDIRawAudioFrame cannot be dropped with a sender as it cannot be received by the sender."
        )
    }

    fn assert_unwritten(&self) {
        assert!(
            self.p_data.is_null(),
            "[Fatal FFI Error] NDIRawAudioFrame data is not null, but should be."
        );
        assert!(
            self.p_metadata.is_null(),
            "[Fatal FFI Error] NDIRawAudioFrame metadata is not null, but should be."
        );
    }
}

unsafe impl Send for NDIRawAudioFrame {}
unsafe impl Sync for NDIRawAudioFrame {}

impl RawFrame for NDIRawAudioFrame {}

pub type AudioFrame = NDIFrame<NDIRawAudioFrame>;

impl AudioFrame {
    pub fn new() -> Self {
        let raw = NDIRawAudioFrame {
            sample_rate: 48_000,
            no_channels: 2,
            no_samples: 0,
            timecode: bindings::NDIlib_send_timecode_synthesize,
            timestamp: 0,
            FourCC: FourCCAudio::FLTP.to_ffi(),
            p_data: std::ptr::null_mut(),
            p_metadata: std::ptr::null_mut(),
            __bindgen_anon_1: bindings::NDIlib_audio_frame_v3_t__bindgen_ty_1 {
                channel_stride_in_bytes: 0,
            },
        };
        Self {
            raw,
            alloc: FrameDataDropGuard::NullPtr,
            custom_state: (),
        }
    }

    pub fn four_cc(&self) -> Option<FourCCAudio> {
        FourCCAudio::from_ffi(self.raw.FourCC)
    }
}
