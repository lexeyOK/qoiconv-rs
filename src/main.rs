use std::fs::File;
use std::io::BufReader;
mod qoi;
use image::{RgbImage, RgbaImage};
use qoi::*;

fn main() {
    let file = File::open("wikipedia_008.qoi").unwrap();
    let bytes = BufReader::new(file);
    let (data, desc) = qoi_decode(bytes, None).unwrap();

    println!("{desc:?}");
    match desc.channels {
        ChanelMode::Rgb => RgbImage::from_raw(desc.width as u32, desc.height as u32, data)
            .unwrap()
            .save("rust.png")
            .unwrap(),
        ChanelMode::Rgba => RgbaImage::from_raw(desc.width as u32, desc.height as u32, data)
            .unwrap()
            .save("rust.png")
            .unwrap(),
    };
}
