extern crate core;

#[cfg(windows)]
use std::fs::File;
use std::io::Write;
use winres::WindowsResource;
use image::ImageReader;

fn main() {
    let img = ImageReader::open("src/cat.png")
        .unwrap()
        .decode()
        .unwrap();
    let rgba = img.to_rgba8();
    let rgba_data = rgba.into_raw();

    let mut file = File::create("icon.rgba").unwrap();
    file.write_all(&rgba_data).unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut res = WindowsResource::new();
        res.set_icon("src/cat.ico");
        res.compile().unwrap();
    }
}