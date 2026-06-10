use uuid::Uuid;

use super::types::{Bucket, UploadResult};
use super::s3_client::S3Storage;

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100 MB
const MAX_IMG_DIMENSION: u32 = 2000;
const THUMB_WIDTH: u32 = 300;
const JPEG_QUALITY: u8 = 85;

struct FileType {
    ext: &'static str,
    mime: &'static str,
    magic: &'static [u8],
    is_image: bool,
}

static ALLOWED_TYPES: &[FileType] = &[
    FileType { ext: "jpg",  mime: "image/jpeg",       magic: &[0xFF, 0xD8, 0xFF], is_image: true },
    FileType { ext: "jpeg", mime: "image/jpeg",       magic: &[0xFF, 0xD8, 0xFF], is_image: true },
    FileType { ext: "png",  mime: "image/png",        magic: &[0x89, 0x50, 0x4E, 0x47], is_image: true },
    FileType { ext: "gif",  mime: "image/gif",        magic: b"GIF8", is_image: true },
    FileType { ext: "webp", mime: "image/webp",       magic: b"RIFF", is_image: true },
    FileType { ext: "pdf",  mime: "application/pdf",  magic: b"%PDF", is_image: false },
];

fn ext_from_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

fn detect_type(bytes: &[u8], claimed_ext: &str) -> Option<&'static FileType> {
    let ext_lower = claimed_ext.to_lowercase();
    ALLOWED_TYPES.iter().find(|t| t.ext == ext_lower && bytes.starts_with(t.magic))
}

/// Process and upload a file to S3
pub async fn upload_file(
    storage: &S3Storage,
    bucket: Bucket,
    bytes: &[u8],
    original_name: &str,
    key_prefix: Option<&str>,
) -> Result<UploadResult, String> {
    if bytes.len() > MAX_FILE_SIZE {
        return Err("File too large (max 100MB)".to_string());
    }

    let claimed_ext = ext_from_name(original_name);
    let ft = detect_type(bytes, claimed_ext)
        .ok_or("Unsupported file type. Allowed: jpg, png, gif, webp, pdf")?;

    let uuid = Uuid::new_v4();
    let key_base = match key_prefix {
        Some(p) => format!("{}/{}", p, uuid),
        None => uuid.to_string(),
    };
    let key = format!("{}.{}", key_base, ft.ext);

    let (processed_bytes, thumb_url) = if ft.is_image {
        let processed = process_image(bytes, ft.mime)?;
        let thumb = generate_thumbnail(bytes, ft.ext)?;

        // Upload thumbnail
        let thumb_key = format!("{}_thumb.{}", key_base, ft.ext);
        let tu = if let Some(thumb_bytes) = thumb {
            let url = storage.put_object(bucket, &thumb_key, thumb_bytes, ft.mime).await?;
            Some(url)
        } else {
            None
        };

        (processed, tu)
    } else {
        (bytes.to_vec(), None)
    };

    let size = processed_bytes.len();
    let url = storage.put_object(bucket, &key, processed_bytes, ft.mime).await?;

    Ok(UploadResult {
        key,
        url,
        thumb_url,
        mime: ft.mime.to_string(),
        size,
        original_name: original_name.to_string(),
    })
}

/// Resize image if too large, re-encode at quality 85 for JPEG
fn process_image(bytes: &[u8], mime: &str) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Invalid image: {}", e))?;
    let (w, h) = (img.width(), img.height());

    // Only process if image needs resizing
    if w <= MAX_IMG_DIMENSION && h <= MAX_IMG_DIMENSION {
        // For JPEG, re-encode at target quality; others pass through
        if mime == "image/jpeg" {
            return encode_jpeg(&img, JPEG_QUALITY);
        }
        return Ok(bytes.to_vec());
    }

    // Resize proportionally
    let resized = img.resize(MAX_IMG_DIMENSION, MAX_IMG_DIMENSION, image::imageops::FilterType::Lanczos3);

    match mime {
        "image/jpeg" => encode_jpeg(&resized, JPEG_QUALITY),
        "image/png" => encode_png(&resized),
        "image/webp" => Ok(bytes.to_vec()), // webp encoding not in image crate defaults
        "image/gif" => Ok(bytes.to_vec()),   // gif resize is lossy, skip
        _ => Ok(bytes.to_vec()),
    }
}

/// Generate a 300px-wide thumbnail
fn generate_thumbnail(bytes: &[u8], ext: &str) -> Result<Option<Vec<u8>>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Invalid image: {}", e))?;
    if img.width() <= THUMB_WIDTH {
        return Ok(None); // Already small enough
    }

    let ratio = THUMB_WIDTH as f64 / img.width() as f64;
    let new_h = (img.height() as f64 * ratio) as u32;
    let thumb = img.resize_exact(THUMB_WIDTH, new_h, image::imageops::FilterType::Lanczos3);

    let encoded = match ext {
        "jpg" | "jpeg" => encode_jpeg(&thumb, JPEG_QUALITY)?,
        "png" => encode_png(&thumb)?,
        _ => return Ok(None),
    };
    Ok(Some(encoded))
}

fn encode_jpeg(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder).map_err(|e| format!("JPEG encode error: {}", e))?;
    Ok(buf.into_inner())
}

fn encode_png(img: &image::DynamicImage) -> Result<Vec<u8>, String> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    img.write_with_encoder(encoder).map_err(|e| format!("PNG encode error: {}", e))?;
    Ok(buf.into_inner())
}
