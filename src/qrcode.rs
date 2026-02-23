use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use qrcodegen::{QrCode, QrCodeEcc};
use skia_safe::{surfaces, Color, Paint, Rect, Image, EncodedImageFormat};



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
    payload_url: &String,
    qr_resolution_px: Option<i32>,
) -> Result<Vec<u8>, &'static str> {

    let qr_resolution_px = match qr_resolution_px {
        Some(qr_resolution_px) =>qr_resolution_px,
        None => 200,
    };

    // QR generation
    let qr = QrCode::encode_text(payload_url, QrCodeEcc::Medium)
        .map_err(|_| "Failed to generate QR")?;

    let modules = qr.size();
    if modules <= 0 {
        return Err("Invalid QR size");
    }

    // Cell size calculation
    let cell_size = qr_resolution_px as f32 / modules as f32;

    // Creating a Skia Surface
    let mut surface = surfaces::raster_n32_premul((qr_resolution_px, qr_resolution_px))
        .ok_or("Failed to create surface")?;
    let canvas = surface.canvas();

    // White background
    let mut paint = Paint::default();
    paint.set_color(Color::WHITE);
    canvas.draw_rect(Rect::from_xywh(0.0, 0.0, qr_resolution_px as f32, qr_resolution_px as f32), &paint);

    // Drawing QR modules
    paint.set_color(Color::BLACK);

    for y in 0..modules {
        for x in 0..modules {
            if qr.get_module(x, y) {
                let px = x as f32 * cell_size;
                let py = y as f32 * cell_size;
                canvas.draw_rect(Rect::from_xywh(px, py, cell_size, cell_size), &paint);
            }
        }
    }

    // PNG export
    let image: Image = surface.image_snapshot();
    #[allow(deprecated)]
    let data = image.encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode PNG")?;

    Ok(data.as_bytes().to_vec())
}
