use std::fs::File;
mod qoi;
use image::{RgbImage, RgbaImage};
use qoi::*;

fn main() {
    let bytes = File::open("qoi_test_images/wikipedia_008.qoi").unwrap();
    let (data, desc) = qoi_decode(bytes, None).unwrap();

    println!("{desc:?}");
    match desc.channels {
        ChanelMode::Rgb => RgbImage::from_raw(desc.width as u32, desc.height as u32, data)
            .unwrap()
            .save("wikipedia_008.png")
            .unwrap(),
        ChanelMode::Rgba => RgbaImage::from_raw(desc.width as u32, desc.height as u32, data)
            .unwrap()
            .save("wikipedia_008.png")
            .unwrap(),
    };
}
