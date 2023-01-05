use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
mod qoi;
use clap::{arg, command, Parser};
use image::RgbaImage;
use qoi::*;

#[derive(Parser)]
#[command(author,version,about,long_about = None)]
struct Args {
    /// Path to input image file
    #[arg(short, long)]
    input: PathBuf,
    /// Path to  qoi output file
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Args::parse();
    match cli.input.extension().and_then(OsStr::to_str) {
        Some("qoi") => {
            let file = File::open(cli.input).expect("cannot open file");
            let buf = BufReader::new(file);
            let (pixels, desc) =
                qoi_decode(buf, Some(ChanelMode::Rgba)).expect("unable to decode qoi image");
            RgbaImage::from_raw(desc.width as u32, desc.height as u32, pixels)
                .expect("unable to encode image")
                .save(&cli.output)
                .unwrap_or_else(|_| panic!("unable to save image to {:?}", cli.output));
        }
        Some(_) => {
            let image = image::open(cli.input).expect("your supplied image is not correct");
            let pixels = image.to_rgba8();
            let mut file = File::create(cli.output).expect("cannot create file");
            let bytes = qoi_encode(
                &pixels,
                QoiDescriptor {
                    width: image.width() as usize,
                    height: image.height() as usize,
                    channels: ChanelMode::Rgba,
                    colorspace: Colorspace::Srgb,
                },
            )
            .expect("unable to decode image");
            file.write_all(&bytes).expect("unable to write to file");
        }
        None => panic!("no extension"),
    }
}
