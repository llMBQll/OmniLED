use image::codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder};
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, ImageFormat};
use std::collections::HashMap;
use std::io::{BufReader, Cursor};

use crate::renderer::buffer::{BitBuffer, BufferTrait};
use crate::script_handler::script_data_types::{ImageData, Size};

pub type CacheKey = (u64, Size, u8);
pub type ImageCache = HashMap<CacheKey, Vec<BitBuffer>>;

pub fn render_image<'a>(
    cache: &'a mut ImageCache,
    image: &ImageData,
    size: Size,
    threshold: u8,
    animated: bool,
) -> &'a Vec<BitBuffer> {
    cache
        .entry((image.hash.unwrap(), size, threshold))
        .or_insert_with(|| {
            if animated {
                render_animated_image(image, size, threshold)
            } else {
                render_static_image(image, size, threshold)
            }
        })
}

fn render_static_image(image: &ImageData, size: Size, threshold: u8) -> Vec<BitBuffer> {
    let image = image::load_from_memory_with_format(&image.bytes, image.format).unwrap();

    vec![render_into_buffer(image, size, threshold)]
}

fn render_animated_image(image: &ImageData, size: Size, threshold: u8) -> Vec<BitBuffer> {
    let reader = BufReader::new(Cursor::new(&image.bytes));

    let frames = match image.format {
        ImageFormat::Png => {
            let decoder = PngDecoder::new(reader).unwrap().apng().unwrap();
            decoder.into_frames()
        }
        ImageFormat::Gif => {
            let decoder = GifDecoder::new(reader).unwrap();
            decoder.into_frames()
        }
        ImageFormat::WebP => {
            let decoder = WebPDecoder::new(reader).unwrap();
            decoder.into_frames()
        }
        _ => panic!("Unsupported image format {:?}", image.format),
    };

    frames
        .collect_frames()
        .unwrap()
        .into_iter()
        .map(|frame| {
            let image = DynamicImage::from(frame.into_buffer());
            render_into_buffer(image, size, threshold)
        })
        .collect()
}

fn render_into_buffer(image: DynamicImage, size: Size, threshold: u8) -> BitBuffer {
    let image = image.resize_exact(size.width as u32, size.height as u32, FilterType::Nearest);
    let image = image.into_luma8();

    let mut buffer = BitBuffer::new(size);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[0] >= threshold {
            buffer.set(x as usize, y as usize);
        }
    }
    buffer
}
