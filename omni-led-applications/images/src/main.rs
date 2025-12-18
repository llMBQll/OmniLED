use clap::{ArgAction, Parser};
use image::guess_format;
use log::{debug, error};
use omni_led_api::plugin::Plugin;
use omni_led_api::types::{ImageData, ImageFormat, Table};

const NAME: &str = "IMAGES";

#[tokio::main]
async fn main() {
    let options = Options::parse();

    // TODO verify that all image names are unique

    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    let images = load_images(options.images);
    plugin.update(images).await.unwrap();
}

fn load_images(image_options: Vec<ImageOptions>) -> Table {
    let mut table = Table::default();

    for option in image_options {
        let (format, bytes) = match load_image(&option.path, &option.format) {
            Ok((format, bytes)) => {
                debug!("Loaded image {:?}", option);
                (format, bytes)
            }
            Err(err) => {
                error!("Failed to load {:?}: {}", option, err);
                continue;
            }
        };

        table.items.insert(
            option.name,
            ImageData {
                format: format as i32,
                data: bytes,
            }
            .into(),
        );
    }

    table
}

fn load_image(
    path: &str,
    format: &Option<String>,
) -> Result<(ImageFormat, Vec<u8>), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;

    let format = match &format {
        Some(format) => match image::ImageFormat::from_extension(format) {
            Some(format) => format,
            None => {
                return Err(format!("Unknown image format '{:?}'", format).into());
            }
        },
        None => guess_format(&bytes)?,
    };

    // Test if image actually loads with provided or guessed format
    let _ = image::load_from_memory_with_format(&bytes, format)?;

    Ok((format.try_into()?, bytes))
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    #[clap(short = 'i', long = "image", action = ArgAction::Append, value_parser = parse_options)]
    images: Vec<ImageOptions>,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct ImageOptions {
    #[clap(index = 1)]
    name: String,

    #[clap(index = 2)]
    path: String,

    #[clap(short, long)]
    format: Option<String>,
}

fn parse_options(
    args: &str,
) -> Result<ImageOptions, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut args = match shlex::split(args) {
        Some(args) => args,
        None => return Err("Failed to parse arguments".into()),
    };
    args.insert(0, "options".into());

    ImageOptions::try_parse_from(args).map_err(|e| e.into())
}
