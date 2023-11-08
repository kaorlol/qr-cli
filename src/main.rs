use rqrr::PreparedImage;
use std::{env, io::Cursor};
use image::io::Reader as ImageReader;
use base64::{Engine as _, engine::general_purpose};

fn download_image(url: &str) -> anyhow::Result<image::DynamicImage> {
    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;
    let reader = ImageReader::new(Cursor::new(bytes));
    Ok(reader.with_guessed_format()?.decode()?)
}

fn load_image(path: &str) -> anyhow::Result<image::DynamicImage> {
    if path.starts_with("http://") || path.starts_with("https://") {
        return download_image(path);
    }

    if path.starts_with("data:image/png;base64,") {
        let base64 = path.replace("data:image/png;base64,", "");
        let data = general_purpose::STANDARD.decode(base64.as_bytes())?;
        let reader = ImageReader::new(Cursor::new(data));
        return Ok(reader.with_guessed_format()?.decode()?);
    }
    
    Ok(image::open(path)?)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let path_usage = format!("{} \"C:\\Users\\user\\Downloads\\qr-code.png\"", args[0]);
        let base64_usage = format!("{} \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZ\"", args[0]);
        let url_usage = format!("{} \"https://example.com/qr-code.png\"", args[0]);
        eprintln!("Usage: {} <image>\nExamples:\n  {}\n  {}\n  {}", args[0], path_usage, base64_usage, url_usage);
        return Ok(());
    }

    let image = load_image(&args[1])?;
    let mut prepared = PreparedImage::prepare(image.into_luma8());
    let grids = prepared.detect_grids();

    if grids.is_empty() {
        eprintln!("No QR codes found in the image");
        return Ok(());
    }

    let (_, content) = grids[0].decode()?;
    println!("{content}");

    Ok(())
}