mod wasapi_audio;
mod cpal_audio;

use std::sync::{Arc, Mutex};
use std::thread::spawn;
use iced::{window, Element, Renderer, Theme, Rectangle, Point, Color, application, Size, Subscription};
use iced::mouse::Cursor;
use iced::widget::{canvas};
use iced::widget::canvas::{Geometry, Stroke, Style};
use wasapi_audio::start_desktop_audio_capture;
use cpal_audio::cpal_audio_capture;

type SharedBuffer = Arc<Mutex<Vec<f32>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ICON_DATA: &[u8] = include_bytes!("../icon.rgba");
    let test = window::icon::from_rgba(Vec::from(ICON_DATA), 90, 90).unwrap();
    hide_console::hide_console();
    let window_size = Size::new(1500.0, 1000.0);
    let setting = window::Settings {
        size: window_size,
        transparent: false,
        resizable: false,
        position: window::Position::Centered,
        icon: test.into(),
        decorations: false,
        //level: window::Level::AlwaysOnTop,
        ..Default::default()
    };
    application("awesome fucking visualizer", Visualizer::update, Visualizer::view)
        .window(setting)
        .subscription(Visualizer::subscription)
        .run_with(|| {
            (Visualizer::new(), window::get_latest().map(Message::GetIced))
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
    raw_id: u64,
}

#[derive(Debug, Clone)]
pub enum Message{
    Tick,
    GetIced(Option<window::Id>),
    GetRaw(u64),
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
            start_desktop_audio_capture(clone_buffer).unwrap_or_else(|_| cpal_audio_capture(clone_buffer2).unwrap());
        });
        Self {
            audio_buffer,
            canvas_cache: canvas::Cache::default(),
            radius: 20.0,
            window_id: window::Id::unique(),
            raw_id: 999999,
        }
    }
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Tick => {
                let buffer = self.audio_buffer.clone();
                let buff = buffer.lock().unwrap();
                let sum_of_absolutes: f32 = buff.iter().map(|&x| (x * x).sqrt()).sum();
                let _ = window::get_latest().and_then(move |id|{println!("{}", id); window::get_minimized(id).into()});
                let rms = (sum_of_absolutes / buff.len() as f32).sqrt();
                self.radius = (exponentiate(rms, -0.5) * 200.0) + 20.0;
                self.canvas_cache.clear();
            },
            Message::GetIced(id) => {
                self.window_id = id.unwrap();
                let _ = window::get_raw_id::<Message>(id.unwrap()).map(Message::GetRaw);
            },
            Message::GetRaw(raw_id) => {
                println!("raw id: {}", raw_id);
                self.raw_id = raw_id;
            }
        }
        iced::Task::none()
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
        let mut frame = canvas::Frame::new(renderer, Size::new(1500.0, 1000.0));
        let test = canvas::Path::circle(Point::new(750.0, 500.0), self.radius * 2.0);
        let buffer = self.buffer.lock().unwrap();
        //print!("\rcurrent radius: {}", self.radius);
        let mut x: f32 = 0.0;
        let test2 = canvas::Path::new(|builder| {
            for coord in buffer[0..2000].iter().step_by(4) {
                x+=3.0;
                let absolute_coord = 500.0 - (coord.clamp(-1.0, 100.0) * 500.0);
                builder.line_to(Point::new(x, absolute_coord));
                builder.move_to(Point::new(x, absolute_coord))
            }
        });
        let blue: f32 = (255.0 - (self.radius) - 20.0 * 2.0).clamp(0.0, 254.0);
        let red: f32 = ((self.radius - 20.0) * 2.0).clamp(0.0, 254.0);
        frame.stroke(&test2, Stroke {
            style: Style::Solid(Color::from_rgb8(red as u8, 0, blue as u8)),
            width: 2.0,
            ..Default::default()
        });
        frame.fill(&test, Color::from_rgb8(red as u8, 0, blue as u8));
        vec![frame.into_geometry()]
    }
}


fn exponentiate(x: f32, k: f32) -> f32 {
    (x * k).exp_m1() / k.exp_m1() // (e ^ (x * k) - 1) / e ^ k - 1
}