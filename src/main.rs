use clap::{arg, command, Parser};
use image::RgbaImage;
use indicatif::{HumanDuration, ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    time::Instant,
};
mod qoi;
use qoi::*;

#[derive(Parser)]
#[command(author,version,about,long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Path to input image files
    input: Vec<PathBuf>,
    /// Directory to output files *UNIMPLEMENTED*
    #[arg(short = 'd', long = "output-dir")]
    output_dir: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    if cli.input.len() == 1 {
        let input = &cli.input[0];
        match input.extension().and_then(OsStr::to_str) {
            Some("qoi") => save_from_qoi(input),
            Some(_) => save_to_qoi(input),
            None => panic!("no extension"),
        };
        println!("done!!");
    } else {
        let started = Instant::now();
        cli.input
            .par_iter()
            .progress_with(
                ProgressBar::new(cli.input.len() as u64).with_style(
                    ProgressStyle::with_template("[{pos}/{len}] [{wide_bar}] {per_sec}")
                        .expect("incorect style")
                        .progress_chars("=> "),
                ),
            )
            .for_each(
                |input: &PathBuf| match input.extension().and_then(OsStr::to_str) {
                    Some("qoi") => save_from_qoi(input),
                    Some(_) => save_to_qoi(input),
                    None => panic!("no extension"),
                },
            );
        println!("Done in {}", HumanDuration(started.elapsed()));
    }
}

fn save_to_qoi(input: &Path) {
    // open and decode image
    let image = image::open(input).expect("your supplied image is not correct");
    let pixels = image.to_rgba8();

    // create file for encoded qoi image
    let mut file = File::create(input.with_extension("qoi")).expect("cannot create file");

    // encode qoi image and write it to file
    let bytes = qoi_encode(
        &pixels,
        &QoiDescriptor {
            width: image.width() as usize,
            height: image.height() as usize,
            channels: ChanelMode::Rgba,
            colorspace: Colorspace::Srgb,
        },
    )
    .expect("unable to decode image");

    file.write_all(&bytes).expect("unable to write to file");
}

fn save_from_qoi(input: &Path) {
    // open file
    let file = File::open(input).expect("cannot open file");
    let buf = BufReader::new(file);

    // decode pixels
    let (pixels, desc) =
        qoi_decode(buf, Some(ChanelMode::Rgba)).expect("unable to decode qoi image");

    // encode in new file and save it
    let output = &input.with_extension("png");
    RgbaImage::from_raw(desc.width as u32, desc.height as u32, pixels)
        .expect("unable to encode image")
        .save(output)
        .unwrap_or_else(|_| panic!("unable to save image to {output:?}"));
}
