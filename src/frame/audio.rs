use super::{NDIFrame, NDIFrameExt, RawFrame, RawFrameInner, drop_guard::FrameDataDropGuard};

pub use crate::bindings::NDIlib_audio_frame_v3_t as NDIRawAudioFrame;
use crate::{bindings, four_cc::FourCCAudio};

impl RawFrameInner for NDIRawAudioFrame {
    #[inline]
    unsafe fn drop_with_recv(&mut self, recv: bindings::NDIlib_recv_instance_t) {
        unsafe { bindings::NDIlib_recv_free_audio_v3(recv, self) }
    }

    #[inline]
    unsafe fn drop_with_sender(&mut self, _sender: bindings::NDIlib_send_instance_t) {
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

impl RawFrame for NDIRawAudioFrame {}

pub type AudioFrame = NDIFrame<NDIRawAudioFrame>;

impl NDIFrameExt<NDIRawAudioFrame> for AudioFrame {
    fn data_valid(&self) -> bool {
        !self.raw.p_data.is_null() && self.alloc != FrameDataDropGuard::NullPtr
    }

    fn clear(&mut self) {
        unsafe { self.drop_buffer_backend() };
        self.raw.p_data = std::ptr::null_mut();
        self.raw.p_metadata = std::ptr::null_mut();
    }
}

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
