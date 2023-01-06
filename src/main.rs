use clap::{arg, command, Parser};
use image::RgbaImage;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
mod qoi;
use qoi::*;

#[derive(Parser)]
#[command(author,version,about,long_about = None, arg_required_else_help = true)]
struct Args {
    /// Path to input image file
    input: PathBuf,
    /// Path to qoi output file
    #[arg(short)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Args::parse();
    match cli.input.extension().and_then(OsStr::to_str) {
        Some("qoi") => {
            // open file
            let file = File::open(&cli.input).expect("cannot open file");
            let buf = BufReader::new(file);

            // decode pixels
            let (pixels, desc) =
                qoi_decode(buf, Some(ChanelMode::Rgba)).expect("unable to decode qoi image");

            // encode in new file and save it
            let output = cli.output.unwrap_or(cli.input.with_extension("png"));
            RgbaImage::from_raw(desc.width as u32, desc.height as u32, pixels)
                .expect("unable to encode image")
                .save(&output)
                .unwrap_or_else(|_| panic!("unable to save image to {:?}", &output));
        }
        Some(_) => {
            // open and decode image
            let image = image::open(&cli.input).expect("your supplied image is not correct");
            let pixels = image.to_rgba8();

            // create file for encoded qoi image
            let output = cli.output.unwrap_or(cli.input.with_extension("qoi"));
            let mut file = File::create(output).expect("cannot create file");

            // encode qoi image and write it to file
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
