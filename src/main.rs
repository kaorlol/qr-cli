use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use image::{io::Reader as ImageReader, DynamicImage};
use rqrr::PreparedImage;
use std::{io::Cursor, path::PathBuf};
use clap::Parser;

#[derive(Parser)]
#[clap(version = "0.2.0", author = "kaoru", about = "A QR code decoder, the cli support url, base64 and local file")]
struct Cli {
    image_path: String,
}

fn grab_image(url: &str) -> anyhow::Result<DynamicImage, anyhow::Error> {
    let response = reqwest::blocking::get(url).context("Failed to make the HTTP request")?;

    let bytes = response.bytes().context("Failed to read response bytes")?;
    let reader = ImageReader::new(Cursor::new(bytes));

    let image = reader
        .with_guessed_format()?
        .decode()
        .context("Failed to decode the image")?;

    Ok(image)
}

fn decode_base64(data: &str) -> anyhow::Result<DynamicImage, anyhow::Error> {
    let prefix_regex = regex::Regex::new(r"data:image/[^;]+;base64,").unwrap();
    let base64 = prefix_regex.replace(data, "");
    let result = general_purpose::STANDARD.decode(base64.as_bytes());
        // .map_err(|_| anyhow::anyhow!("Failed to decode the base64 path"))?;
    
    if result.is_ok() {
        let reader = ImageReader::new(Cursor::new(result?));
        let image = reader.with_guessed_format()?
            .decode()
            .context("Failed to decode the image")?;

        return Ok(image);
    }

    Err(anyhow::anyhow!("Failed to decode the base64 path"))
}

fn load_image(path: &str) -> Result<DynamicImage, anyhow::Error> {
    let os_path = PathBuf::from(path);

    if os_path.exists() {
        return Ok(image::open(os_path)?);
    }

    if path.contains("://") {
        return grab_image(path);
    }

    match decode_base64(path) {
        Ok(image) => return Ok(image),
        Err(_) => {}
    }

    Err(anyhow::anyhow!("Input is not a valid path, URL or base64 string"))
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let image = load_image(&args.image_path)?;

    let mut prepared = PreparedImage::prepare(image.into_luma8());
    let grids = prepared.detect_grids();

    assert!(!grids.is_empty(), "No QR codes found in the image");

    let (_, content) = grids[0].decode()?;
    println!("{}", content);

    Ok(())
}