/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::{ArgAction, Parser};
use image::{ImageBuffer, ImageFormat, Luma};
use log::error;
use omni_led_api::plugin::Plugin;
use omni_led_api::types::{Image, Table};
use std::path::{Path, PathBuf};

const NAME: &str = "IMAGES";

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    let images = load_images(options.image_options);
    plugin.update(images).await.unwrap();
}

fn load_images(image_options: Vec<ImageOptions>) -> Table {
    let mut table = Table::default();

    for option in image_options {
        let image = match load_image(&option.path, option.threshold, option.format) {
            Ok(image) => image,
            Err(err) => {
                error!("Failed to load {:?}: {}", option, err);
                continue;
            }
        };
        table.items.insert(option.name, image.into());
    }

    table
}

fn load_image(
    path: &Path,
    threshold: u8,
    format: Option<ImageFormat>,
) -> Result<Image, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;

    let image = match format {
        Some(format) => image::load_from_memory_with_format(&bytes, format)?,
        None => image::load_from_memory(&bytes)?,
    };

    let image = image.into_luma8();

    let image: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
            let pixel = image.get_pixel(x, y);
            if pixel[0] > threshold {
                Luma([255])
            } else {
                Luma([0])
            }
        });

    let image = Image {
        width: image.width() as i64,
        height: image.height() as i64,
        data: image.into_raw(),
    };
    Ok(image)
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    #[clap(short = 'i', long = "image", action = ArgAction::Append, value_parser = parse_image_options)]
    image_options: Vec<ImageOptions>,
}

#[derive(Debug, Clone)]
struct ImageOptions {
    name: String,
    path: PathBuf,
    threshold: u8,
    format: Option<ImageFormat>,
}

fn parse_image_options(
    s: &str,
) -> Result<ImageOptions, Box<dyn std::error::Error + Send + Sync + 'static>> {
    // TODO better option format

    let args = match shlex::split(s) {
        Some(args) => args,
        None => return Err("Failed to parse arguments".into()),
    };

    if args.len() < 3 || args.len() > 4 {
        return Err("Expected options: NAME PATH THRESHOLD [FORMAT]".into());
    }

    let name = args[0].clone();
    let path = PathBuf::from(&args[1]);
    let threshold = args[2].parse::<u8>()?;
    let format = if args.len() == 4 {
        let format = ImageFormat::from_extension(&args[3]);
        if let None = format {
            error!(
                "Failed to parse image format from '{}', continuing anyways",
                args[3]
            );
        }
        format
    } else {
        None
    };

    Ok(ImageOptions { name, path, threshold, format })
}
