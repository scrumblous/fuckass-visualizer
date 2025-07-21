use std::sync::{Arc, Mutex};
use windows::{
    Win32::Media::Audio::*,
    Win32::System::Com::*,
};
use crate::SharedBuffer;

// AI GENERATED BECAUSE WINAPI SUCKS ASS SORRY
pub fn start_desktop_audio_capture(buffer: SharedBuffer) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Initialize COM
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        // Create device enumerator
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        // Get default audio endpoint (speakers/headphones)
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // Activate audio client
        let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

        // Get the default format
        let wave_format = audio_client.GetMixFormat()?;

        // Initialize in loopback mode
        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            10_000_000, // 1 second buffer
            0,
            wave_format,
            None,
        )?;

        // Get capture client
        let capture_client: IAudioCaptureClient = audio_client.GetService()?;

        audio_client.Start()?;

        println!("Desktop audio capture started successfully\n");

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10)); // Small delay to prevent busy waiting

            // Get available frames
            let packet_length = capture_client.GetNextPacketSize()?;

            if packet_length > 0 {
                // Get the audio data
                let mut data_ptr = std::ptr::null_mut();
                let mut num_frames = 0u32;
                let mut flags = 0u32;

                capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    None,
                    None,
                )?;

                if num_frames > 0 && !data_ptr.is_null() {
                    // Convert to f32 samples (assuming 32-bit float format)
                    let channels = (*wave_format).nChannels as usize;
                    let sample_count = (num_frames as usize) * channels;
                    let samples = std::slice::from_raw_parts(data_ptr as *const f32, sample_count);

                    if let Ok(mut buffer_guard) = buffer.try_lock() {
                        buffer_guard.extend_from_slice(samples);
                        let buffer_len = buffer_guard.len();
                        if buffer_len > 2000 {
                            buffer_guard.drain(0..buffer_len - 2000);
                        }
                    }
                }

                // Release the buffer
                capture_client.ReleaseBuffer(num_frames)?;
            }
        }
    }
}