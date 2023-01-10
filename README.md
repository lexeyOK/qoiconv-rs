# qoi-test
Attempt to port [`qoi.h`](https://qoiformat.org/) to rust-lang.
## Usage
Copy qoi.rs to your source file and add it as a mod.

Example of decoding pixels from `.qoi` file:
```rust 
use std::fs::File;
use std::io::BufReader;
mod qoi;
use qoi::*;

fn main() {
    // load file and get bytes (use `BufReader` to speed up reads)
    let file = File::open("wikipedia_008.qoi").unwrap();
    let mut bytes = BufReader::new(file);

    // get pixels and descriptor
    let (data, desc) = qoi_decode(bytes, None).unwrap();
}

```

Example of encoding pixels into `.qoi` file:
```rust
use std::fs::File;
use std::io::Write;
mod qoi;
use qoi::*;
fn main() {
    // get pixels and make valid descriptor
    // pixels must be laid out in order RGB(A)
    let pixels = [255, 0, 0, 15, 1, 255, 255, 255, 191, 255, 0, 0, 15, 1, 74];
    let desc = QoiDescriptor {
        width: pixels.len() / 3,
        height: 1,
        channels: ChanelMode::Rgb,
        colorspace: Colorspace::Linear,
    };

    let bytes = qoi_encode(&pixels, &desc).unwrap();

    let mut f = File::create("example.qoi").unwrap();
    f.write_all(bytes.as_slice()).unwrap();
}
```
## Testing, fuzzing, benches, profile
Use cargo to test and fuzz the program (you'll need to install cargo-fuzz):
```bash
cargo test
# install cargo-fuzz with `cargo install cargo-fuzz`
cargo fuzz run qoi-test-pixels
```
To run benches against c test program run:
> TODO implement new benchmark to compare against c 

This implementation is 2x slower then c implementation probably due to smaller buffer size of `BufReader`.
## Usage of qoiconv-rs
```bash
# convert form image to qoi
qoiconv-rs -i input.png -o output.qoi
# convert from qoi to image 
qoiconv-rs -i input.qoi -o output.png
```
