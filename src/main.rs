use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use iced::{window, Element, Renderer, Theme, Rectangle, Point, Color, application, Size, Subscription, mouse, Event};
use iced::mouse::Cursor;
use iced::widget::{canvas};
use iced::widget::canvas::{Geometry, Stroke, Style};

use windows::{
    Win32::Media::Audio::*,
    Win32::System::Com::*,
};

type SharedBuffer = Arc<Mutex<Vec<f32>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ICON_DATA: &[u8] = include_bytes!("../icon.rgba");
    let test = window::icon::from_rgba(Vec::from(ICON_DATA), 90, 90).unwrap();
    hide_console::hide_console();
    let window_size = Size::new(1000.0, 500.0);
    let setting = window::Settings {
        size: window_size,
        transparent: false,
        resizable: false,
        position: window::Position::Centered,
        icon: test.into(),
        decorations: false,
        ..Default::default()
    };
    application("awesome fucking visualizer", Visualizer::update, Visualizer::view)
        .window(setting)
        .subscription(Visualizer::subscription)
        .run_with(|| {
            (Visualizer::new(), window::get_latest().map(Message::Testing))
        }
    ).expect("some error");
    Ok(())
}

#[derive(Debug)]
pub struct Visualizer {
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    canvas_cache: canvas::Cache,
    radius: f32,
    window_id: window::Id,
}

#[derive(Debug, Clone)]
pub enum Message{
    Tick,
    Testing(Option<window::Id>),
}

pub struct AudioCanvas{
    buffer: SharedBuffer,
    radius: f32,
}

impl Visualizer {
    fn new() -> Self{
        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let clone_buffer = audio_buffer.clone();
        let clone_buffer2 = audio_buffer.clone();
        spawn(move || {
            start_desktop_audio_capture(clone_buffer).unwrap_or_else(|_| start_audio_capture(clone_buffer2).unwrap());
        });
        Self {
            audio_buffer,
            canvas_cache: canvas::Cache::default(),
            radius: 20.0,
            window_id: window::Id::unique(),
        }
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let buffer = self.audio_buffer.clone();
                let buff = buffer.lock().unwrap();
                let sum_of_absolutes: f32 = buff.iter().map(|&x| (x * x).sqrt()).sum();
                let _ = window::get_latest().and_then(move |id|{println!("{}", id); window::get_minimized(id).into()});
                let rms = (sum_of_absolutes / buff.len() as f32).sqrt();
                self.radius = (push_towards_extreme(rms, 2.0) * 200.0) + 20.0;
                self.canvas_cache.clear();
            },
            Message::Testing(id) => {
                self.window_id = id.unwrap();
            }
        }
    }
    fn view(&self) -> Element<'_, Message> {
        canvas(AudioCanvas {buffer: self.audio_buffer.clone(), radius: self.radius}).into()
    }
    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick)
    }
}

impl <Message> canvas::Program<Message> for AudioCanvas{
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, _bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, Size::new(1000.0, 500.0));
        let test = canvas::Path::circle(Point::new(500.0, 0.0), self.radius);
        let buffer = self.buffer.lock().unwrap();
        //print!("\rcurrent radius: {}", self.radius);
        let mut x: f32 = 0.0;
        let test2 = canvas::Path::new(|builder| {
            for coord in buffer[0..2000].iter().step_by(4) {
                x+=2.0;
                let absolute_coord = 500.0 - ((coord * coord).sqrt() * 500.0);
                builder.line_to(Point::new(x, absolute_coord));
                builder.move_to(Point::new(x, absolute_coord))
            }
        });
        let blue: f32 = (255.0 - self.radius * 2.0).clamp(0.0, 254.0);
        let red: f32 = (self.radius * 2.0).clamp(0.0, 254.0);
        frame.stroke(&test2, Stroke {
            style: Style::Solid(Color::from_rgb8(red as u8, 0, blue as u8)),
            width: 2.0,
            ..Default::default()
        });
        frame.fill(&test, Color::from_rgb8(red as u8, 0, blue as u8));
        vec![frame.into_geometry()]
    }
}
// OLD WAY OF CAPTURING AUDIO (VIRTUAL CABLE :SOB:)
fn start_audio_capture(buffer: SharedBuffer) -> Result<(), Box<dyn std::error::Error>> {
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

    // keep the audio thread alive
    loop {
        //println!("buffer length: {:?}", bufclone.lock().unwrap().len());
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// AI GENERATED BECAUSE WINAPI SUCKS
fn start_desktop_audio_capture(buffer: SharedBuffer) -> Result<(), Box<dyn std::error::Error>> {
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

        // Start the stream
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

                    // Update the shared buffer (same logic as your cpal function)
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


fn push_towards_extreme(x: f32, strength: f32) -> f32 {
    if x < 0.5 {
        ((2.0 * x).powf(strength)) / 2.0
    } else {
        1.0 - ((2.0 * (1.0 - x)).powf(strength)) / 2.0
    }
}
