use std::io::prelude::*;

///  The qoi_desc describes the input pixel data.
#[derive(Clone, PartialEq, Debug)]
pub struct QoiDescriptor {
    pub width: usize,
    pub height: usize,
    pub channels: ChanelMode,
    pub colorspace: Colorspace,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChanelMode {
    Rgb = 3,
    Rgba = 4,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Colorspace {
    Srgb = 0,
    Linear = 1,
}

#[derive(Copy, Clone, PartialEq)]
struct QoiRGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl QoiRGBA {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

const QOI_OP_INDEX: u8 = 0x00; /* 00xxxxxx */
const QOI_OP_DIFF: u8 = 0x40; /* 01xxxxxx */
const QOI_OP_LUMA: u8 = 0x80; /* 10xxxxxx */
const QOI_OP_RUN: u8 = 0xc0; /* 11xxxxxx */
const QOI_OP_RGB: u8 = 0xfe; /* 11111110 */
const QOI_OP_RGBA: u8 = 0xff; /* 11111111 */

const QOI_MASK: u8 = 0xc0; /* 11000000 */

const fn color_hash(pixel: QoiRGBA) -> usize {
    let QoiRGBA { r, g, b, a } = pixel;
    r as usize * 3 + g as usize * 5 + b as usize * 7 + a as usize * 11
}

const QOI_HEADER_SIZE: usize = 14;

/* 2GB is the max file size that this implementation can safely handle. We guard
against anything larger than that, assuming the worst case with 5 bytes per
pixel, rounded down to a nice clean value. 400 million pixels ought to be
enough for anybody. */
const QOI_PIXELS_MAX: usize = 400_000_000;

const QOI_PADDING_SIZE: usize = 8;
const QOI_PADDING: [u8; QOI_PADDING_SIZE] = [0, 0, 0, 0, 0, 0, 0, 1];

/// Encode raw RGB or RGBA pixels into a QOI image in memory.
pub fn qoi_encode(
    pixels: &[u8],
    desc: QoiDescriptor,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    //int i, p, run;
    assert_eq!(
        pixels.len(),
        desc.width * desc.height * (desc.channels as usize)
    );

    if desc.width == 0 || desc.height == 0 {
        return Err("zero width or height".into());
    }

    if desc.height >= QOI_PIXELS_MAX / desc.width {
        return Err("exceeded maximum safe pixel count".into());
    }

    let max_size = desc.width * desc.height * (desc.channels as usize + 1)
        + QOI_HEADER_SIZE
        + QOI_PADDING_SIZE;
    let mut bytes = Vec::with_capacity(max_size);

    bytes.write_all(b"qoif")?;
    bytes.write_all(&(desc.width as u32).to_be_bytes())?;
    bytes.write_all(&(desc.height as u32).to_be_bytes())?;
    bytes.write_all(&[desc.channels as u8, desc.colorspace as u8])?;
    
    let mut pixel_previous = QoiRGBA::new(0, 0, 0, 255);
    
    let mut index = [QoiRGBA::new(0, 0, 0, 0); 64];

    let pixel_end = pixels.len() - desc.channels as usize;

    let mut run = 0;
    for pixel_pos in (0..pixels.len()).step_by(desc.channels as usize) {
        let pixel = if desc.channels == ChanelMode::Rgba {
            QoiRGBA::new(
                pixels[pixel_pos],
                pixels[pixel_pos + 1],
                pixels[pixel_pos + 2],
                pixels[pixel_pos + 3],
            )
        } else {
            QoiRGBA::new(
                pixels[pixel_pos],
                pixels[pixel_pos + 1],
                pixels[pixel_pos +2],
                255,
            )
        };
        if pixel == pixel_previous {
            run += 1;
            if run == 62 || pixel_pos == pixel_end {
                bytes.write_all(&[QOI_OP_RUN | (run - 1)])?;

                run = 0;
            }
        } else {
            if run > 0 {
                bytes.write_all(&[QOI_OP_RUN | (run - 1)])?;

                run = 0;
            }

            let index_pos = color_hash(pixel) % 64;

            if index[index_pos] == pixel {
                bytes.write_all(&[QOI_OP_INDEX | index_pos as u8])?;
            } else {
                index[index_pos] = pixel;

                if pixel.a == pixel_previous.a {
                    let dr = pixel.r.wrapping_sub(pixel_previous.r) as i8;
                    let dg = pixel.g.wrapping_sub(pixel_previous.g) as i8;
                    let db = pixel.b.wrapping_sub(pixel_previous.b) as i8;

                    let dg_dr = dr.wrapping_sub(dg);
                    let dg_db = db.wrapping_sub(dg);

                    if (-2..=1).contains(&dr) && (-2..=1).contains(&dg) && (-2..=1).contains(&db) {
                        bytes.write_all(&[QOI_OP_DIFF
                            | ((dr + 2) as u8) << 4
                            | ((dg + 2) as u8) << 2
                            | ((db + 2) as u8)])?;
                    } else if (-8..=7).contains(&dg_dr)
                        && (-8..=7).contains(&dg_db)
                        && (-32..=31).contains(&dg)
                    {
                        bytes.write_all(&[
                            QOI_OP_LUMA | ((dg + 32) as u8),
                            ((dg_dr + 8) as u8) << 4 | ((dg_db + 8) as u8),
                        ])?;
                    } else {
                        bytes.write_all(&[QOI_OP_RGB, pixel.r, pixel.g, pixel.b])?;
                    }
                } else {
                    bytes.write_all(&[QOI_OP_RGBA, pixel.r, pixel.g, pixel.b, pixel.a])?;
                }
            }
        }

        pixel_previous = pixel;
    }
    bytes.write_all(&QOI_PADDING)?;
    bytes.flush()?;
    Ok(bytes)
}

/// Decode a QOI image from memory.
pub fn qoi_decode(
    mut data: impl Read,
    channels: Option<ChanelMode>,
) -> Result<(Vec<u8>, QoiDescriptor), Box<dyn std::error::Error>> {
    let mut u32_buf = [0u8; 4];
    let mut u8_buf = [0u8; 1];
    macro_rules! read_u32 {
        () => {{
            data.read_exact(&mut u32_buf)?;
            u32::from_be_bytes(u32_buf)
        }};
    }
    macro_rules! read_u8 {
        () => {{
            data.read_exact(&mut u8_buf)?;
            u8_buf[0]
        }};
    }

    let mut header_magic: [u8; 4] = [0; 4];
    data.read_exact(&mut header_magic)?;

    if u32::from_be_bytes(header_magic) != u32::from_be_bytes(*b"qoif") {
        return Err(format!("unexpected header: {header_magic:?}").into());
    }

    let width = read_u32!() as usize;
    let height = read_u32!() as usize;

    let channels = match channels {
        Some(channel) => {
            read_u8!();
            channel
        }
        None => match read_u8!() {
            3 => ChanelMode::Rgb,
            4 => ChanelMode::Rgba,
            _ => {
                return Err("unexpected number of color channels".into());
            }
        },
    };

    let colorspace = match read_u8!() {
        0 => Colorspace::Srgb,
        1 => Colorspace::Linear,
        _ => {
            return Err("unexpected colorspace".into());
        }
    };

    let desc = QoiDescriptor {
        width,
        height,
        channels,
        colorspace,
    };

    if desc.width == 0 || desc.height == 0 {
        return Err("width or height is zero".into());
    }

    if desc.height >= QOI_PIXELS_MAX / desc.width {
        return Err("exceeded maximum safe pixel count".into());
    }

    let pixel_len = desc.width * desc.height * (channels as usize);
    let mut pixels = Vec::with_capacity(pixel_len);

    let mut index = [QoiRGBA::new(0, 0, 0, 0); 64];
    let mut pixel = QoiRGBA::new(0, 0, 0, 0);

    let mut run = 0;
    for _ in (0..pixel_len).step_by(channels as usize) {
        if run > 0 {
            run -= 1;
        } else {
            let b1 = read_u8!();

            if b1 == QOI_OP_RGB {
                pixel.r = read_u8!();
                pixel.g = read_u8!();
                pixel.b = read_u8!();
            } else if b1 == QOI_OP_RGBA {
                pixel.r = read_u8!();
                pixel.g = read_u8!();
                pixel.b = read_u8!();
                pixel.a = read_u8!();
            } else if (b1 & QOI_MASK) == QOI_OP_INDEX {
                pixel = index[b1 as usize];
            } else if (b1 & QOI_MASK) == QOI_OP_DIFF {
                let dr = ((b1 >> 4) & 0x03) as i8 - 2;
                let dg = ((b1 >> 2) & 0x03) as i8 - 2;
                let db = (b1 & 0x03) as i8 - 2;
                pixel.r = pixel.r.wrapping_add_signed(dr);
                pixel.g = pixel.g.wrapping_add_signed(dg);
                pixel.b = pixel.b.wrapping_add_signed(db);
            } else if (b1 & QOI_MASK) == QOI_OP_LUMA {
                let b2 = read_u8!();

                let dg = (b1 & 0x3f) as i8 - 32;
                let dr = dg - 8 + ((b2 >> 4) & 0x0f) as i8;
                let db = dg - 8 + (b2 & 0x0f) as i8;

                pixel.r = pixel.r.wrapping_add_signed(dr);
                pixel.g = pixel.g.wrapping_add_signed(dg);
                pixel.b = pixel.b.wrapping_add_signed(db);
            } else if (b1 & QOI_MASK) == QOI_OP_RUN {
                run = b1 & 0x3f;
            }

            index[color_hash(pixel) % 64] = pixel;
        }

        pixels.push(pixel.r);
        pixels.push(pixel.g);
        pixels.push(pixel.b);

        if channels as usize == 4 {
            pixels.push(pixel.a);
        }
    }

    Ok((pixels, desc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;
    use std::io::Cursor;
    quickcheck! {
        fn inverse_application(xs: Vec<u8>) ->bool{
            if xs.len()!=100{
                return true;
            }
            let desc = QoiDescriptor{
                width: 10,
                height: 10,
                channels: ChanelMode::Rgb,
                colorspace: Colorspace::Linear
            };
            let bytes = qoi_encode(&xs,desc).unwrap();
            let (pixels, _) = qoi_decode(Cursor::new(bytes),None).unwrap();

            xs == pixels
        }
    }
    #[test]
    fn inverse_application_test() {
        let pixels = [1, 0, 0, 0, 0, 0];
        let desc = QoiDescriptor {
            width: pixels.len() / 3,
            height: 1,
            channels: ChanelMode::Rgb,
            colorspace: Colorspace::Linear,
        };
        let bytes = qoi_encode(&pixels, desc.clone()).unwrap();
        dbg!(&bytes);
        let (pixels_, _desc) = qoi_decode(Cursor::new(bytes), None).unwrap();
        dbg!(&pixels_);
        assert_eq!(pixels_, pixels);
    }
    #[test]
    fn first_pixel_zero() {
        let pixels = [0,0,0,0, 0, 1];
        let desc = QoiDescriptor {
            width: pixels.len() / 3,
            height: 1,
            channels: ChanelMode::Rgb,
            colorspace: Colorspace::Linear,
        };
        let bytes = qoi_encode(&pixels, desc.clone()).unwrap();
        dbg!(&bytes);
        let (pixels_decoded, _desc) = qoi_decode(Cursor::new(bytes), None).unwrap();
        dbg!(&pixels_decoded);
        assert_eq!(pixels_decoded, pixels);
    }
}
