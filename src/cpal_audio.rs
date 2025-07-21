use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::SharedBuffer;

// OLD WAY OF CAPTURING AUDIO (VIRTUAL CABLE :SOB:)
pub fn cpal_audio_capture(buffer: SharedBuffer) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device found");
    let config = device.default_input_config()?;
    let bufclone = buffer.clone();

    println!("Starting audio capture from: {}", device.name()?);

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _info| {
            if let Ok(mut buffer) = buffer.try_lock() {
                buffer.extend_from_slice(data);
                let bufferlen = buffer.len();
                if buffer.len() > 2000 {
                    buffer.drain(0..bufferlen - 2000);
                }
            }
            let buf = bufclone.lock().unwrap();
            let sum_of_absolutes: f32 = buf.iter().map(|&x| (x * x).sqrt()).sum();
            let rms = sum_of_absolutes / buf.len() as f32;
            print!("\raverage volume: {rms}");
        },
        |err| eprintln!("Audio error: {}", err),
        None,
    )?;

    stream.play()?;

    loop {
        //println!("buffer length: {:?}", bufclone.lock().unwrap().len());
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}