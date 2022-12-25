use std::{fs::File, io::Write};
mod qoi;
use qoi::{qoi_encode, ChanelMode, Colorspace, QoiDescriptor};

fn main() {
    let image = image::open("qoi_test_images/dice.png").unwrap();

    let pixels = image.as_bytes();

    let mut data = qoi_encode(
        pixels,
        QoiDescriptor {
            width: image.width() as usize,
            height: image.height() as usize,
            channels: ChanelMode::Rgba,
            colorspace: Colorspace::Linear,
        },
    )
    .unwrap();

    let mut image = File::create("dice.qoi").unwrap();
    image.write_all(&mut data).unwrap();
}
