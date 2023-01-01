#![no_main]
mod qoi;
use qoi::*;
use std::io::Cursor;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|pixels: &[u8]| {
    // fuzzed code goes
    if pixels.len() % 3 != 0 || pixels.len() < 3 {
        return;
    }
    let desc = QoiDescriptor {
        width: pixels.len() / 3,
        height: 1,
        channels: ChanelMode::Rgb,
        colorspace: Colorspace::Linear,
    };
    let bytes = qoi_encode(&pixels, desc.clone()).unwrap();
    let (pixels_, _desc) = qoi_decode(Cursor::new(bytes), None).unwrap();
    assert_eq!(pixels_, pixels);
});
