use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use image::{io::Reader as ImageReader, DynamicImage};
use rqrr::PreparedImage;
use std::path::PathBuf;
use std::{env, io::Cursor};

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

fn decode_base64_path(path: &str) -> Result<image::DynamicImage, anyhow::Error> {
    let base64_regex = regex::Regex::new(r#"^data:image/[^;]+;base64,([A-Za-z0-9+/]+={0,2})$"#).unwrap();
    if let Some(captures) = base64_regex.captures(path) {
        let base64_part = captures.get(1).unwrap().as_str();

        let data = general_purpose::STANDARD.decode(base64_part.as_bytes())
            .map_err(|_| anyhow::anyhow!("Failed to decode the base64 path"))?;
        
        let reader = ImageReader::new(std::io::Cursor::new(data));
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

    if path.starts_with("data:image/") {
        return decode_base64_path(path);
    }

    Ok(image::open(path)?)
}

// TODO: Use clap for argument parsing
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let image_path = &args[1];

    if args.len() < 2 {
        let path_usage = format!("{} \"C:\\Users\\user\\Downloads\\qr-code.png\"", args[0]);
        let base64_usage = format!("{} \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZ\"", args[0]);
        let url_usage = format!("{} \"https://example.com/qr-code.png\"", args[0]);
        eprintln!(
            "Usage: {} <image>\nExamples:\n  {}\n  {}\n  {}",
            args[0], path_usage, base64_usage, url_usage
        );
        return Ok(());
    }

    let image = load_image(image_path)?;

    let mut prepared = PreparedImage::prepare(image.into_luma8());
    let grids = prepared.detect_grids();

    assert!(!grids.is_empty(), "No QR codes found in the image");

    let (_, content) = grids[0].decode()?;
    println!("{content}");

    Ok(())
}
