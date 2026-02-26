use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use qrcode::QrCode;
use image::{ImageBuffer, Luma};
use std::io::Cursor;

pub(crate) fn decode_base64_or_base64url(input: &str) -> Result<Vec<u8>, &'static str> {
    if input.trim().is_empty() {
        return Err("invoiceHash is empty.");
    }

    let mut s = input.trim().to_string();
    if s.contains('-') || s.contains('_') {
        s = s.replace('-', "+").replace('_', "/");
    }

    match s.len() % 4 {
        0 => {}
        2 => s.push_str("=="),
        3 => s.push_str("="),
        _ => return Err("Invalid Base64/Base64Url length."),
    }

    general_purpose::STANDARD
        .decode(s)
        .map_err(|_| "Invalid Base64/Base64Url")
}


pub(crate) fn build_invoice_verification_url(
    base_url: &String,
    nip: &String,
    issue_date:&DateTime<Utc>,
    invoice_hash: &String,
) -> Result<String, &'static str> {

    let base_url = base_url
            .trim_end_matches('/')
            .to_string();

    let date = issue_date.format("%d-%m-%Y").to_string();

    let decoded = decode_base64_or_base64url(invoice_hash)
        .map_err(|_| "Invalid Base64/Base64Url")?;

    let encoded = general_purpose::URL_SAFE_NO_PAD.encode(decoded);

    Ok(format!("{}/invoice/{}/{}/{}", base_url, nip, date, encoded))
}

pub(crate) fn generate(
    data: &String,
    qr_resolution_px: Option<u32>,
) -> Result<Vec<u8>, &'static str> {

    let qr_resolution_px = match qr_resolution_px {
        Some(qr_resolution_px) =>qr_resolution_px,
        None => 200,
    };

    let qr = QrCode::new(data).map_err(|_| "Failed to generate QR")?;
    let modules = qr.width() as u32;

    if modules == 0 {
        return Err("Invalid QR size");
    }

    let cell_size = qr_resolution_px as f32 / modules as f32;

    let mut img = ImageBuffer::<Luma<u8>, Vec<u8>>::new(qr_resolution_px, qr_resolution_px);

    for pixel in img.pixels_mut() {
        *pixel = Luma([255u8]);
    }

    for y in 0..modules {
        for x in 0..modules {
            if qr[(x as usize, y as usize)] == qrcode::Color::Dark {
                let px = (x as f32 * cell_size).floor() as u32;
                let py = (y as f32 * cell_size).floor() as u32;

                let w = cell_size.ceil() as u32;
                let h = cell_size.ceil() as u32;

                for dy in 0..h {
                    for dx in 0..w {
                        let xx = px + dx;
                        let yy = py + dy;

                        if xx < qr_resolution_px && yy < qr_resolution_px {
                            img.put_pixel(xx, yy, Luma([0u8]));
                        }
                    }
                }
            }
        }
    }

    // Save to PNG
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageOutputFormat::Png)
        .map_err(|_| "Failed to encode PNG")?;

    Ok(buf.into_inner())
}
