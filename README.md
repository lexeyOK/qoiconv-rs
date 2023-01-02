# qoi-test
Attempt to port [`qoi.h`](qoi-image.org) to rust-lang.
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

    let bytes = qoi_encode(&pixels, desc).unwrap();

    let mut f = File::create("example.qoi").unwrap();
    f.write_all(bytes.as_slice()).unwrap();
}
```
## testing, fuzzing, benches, profile
Use cargo to test and fuzz the program (you'll need to install cargo-fuzz):
```bash
# install cargo-fuzz with `cargo install cargo-fuzz`
cargo fuzz run qoi-test-pixels
cargo test
```
To run benches against c test program run:
```bash
cp qoi_test_images/wikipedia_008.qoi test/
# compile c program with gcc
gcc cc/qoi-test.c -std=c99 -O3 -o test/qoi-test-c
# compile rust program with cargo
cargo b -r && mv target/release/qoi-test test/qoi-test-rust 
# run hyperfine or other benchmark tool
hyperfine ./qoi-test-c ./qoi-test-rust
# run cargo-flamegraph 
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --root
```
This implementation is 2x slower then c implementation probably due to smaller buffer size of `BufReader`.