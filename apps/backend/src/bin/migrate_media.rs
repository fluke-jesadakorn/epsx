/// One-time migration: filesystem → MinIO
///
/// Reads files from:
///   - /data/chat_uploads/   (Docker volume, chat attachments)
///   - /tmp/news_uploads/    (ephemeral, news cover images — may be empty)
///
/// Uploads each to the appropriate MinIO bucket, preserving filenames.
/// Also patches existing DB rows to use new CDN URLs.
///
/// Usage:
///   cargo run --bin migrate_media
///
/// Environment:
///   MINIO_ENDPOINT, MINIO_ACCESS_KEY, MINIO_SECRET_KEY, MINIO_PUBLIC_URL, DATABASE_URL

use std::env;
use std::path::Path;

use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    primitives::ByteStream,
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    epsx::config::env::load_env();

    let endpoint = env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT required");
    let access = env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY required");
    let secret = env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY required");
    let public_url = env::var("MINIO_PUBLIC_URL").unwrap_or_else(|_| endpoint.clone());

    let creds = Credentials::new(&access, &secret, None, None, "minio");
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(&endpoint)
        .credentials_provider(creds)
        .force_path_style(true)
        .build();
    let client = Client::from_conf(config);

    // Ensure buckets exist
    for bucket in ["chat", "news", "notifications", "public"] {
        if client.head_bucket().bucket(bucket).send().await.is_err() {
            client.create_bucket().bucket(bucket).send().await?;
            println!("Created bucket: {}", bucket);
        }
    }

    let mut success = 0u64;
    let mut fail = 0u64;

    // --- Migrate news uploads ---
    let news_dir = env::var("NEWS_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/news_uploads".to_string());
    if Path::new(&news_dir).exists() {
        println!("\n=== Migrating news uploads from {} ===", news_dir);
        let (s, f) = migrate_dir(&client, &news_dir, "news").await;
        success += s;
        fail += f;
    } else {
        println!("News upload dir {} does not exist, skipping", news_dir);
    }

    // --- Migrate chat uploads ---
    let chat_dir = env::var("CHAT_UPLOAD_DIR").unwrap_or_else(|_| "/data/chat_uploads".to_string());
    if Path::new(&chat_dir).exists() {
        println!("\n=== Migrating chat uploads from {} ===", chat_dir);
        let (s, f) = migrate_chat_dir(&client, &chat_dir).await;
        success += s;
        fail += f;
    } else {
        println!("Chat upload dir {} does not exist, skipping", chat_dir);
    }

    println!("\n=== Migration complete ===");
    println!("Uploaded: {}, Failed: {}", success, fail);

    // --- Patch DB cover_image_url values ---
    if let Ok(db_url) = env::var("DATABASE_URL") {
        println!("\n=== Patching news_articles cover_image_url → CDN URLs ===");
        patch_news_urls(&db_url, &public_url)?;
    }

    Ok(())
}

async fn migrate_dir(client: &Client, dir: &str, bucket: &str) -> (u64, u64) {
    let mut success = 0u64;
    let mut fail = 0u64;

    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Cannot read {}: {}", dir, e);
            return (0, 0);
        }
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_file() { continue; }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let mime = mime_from_ext(&filename);
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                match client
                    .put_object()
                    .bucket(bucket)
                    .key(&filename)
                    .body(ByteStream::from(bytes))
                    .content_type(mime)
                    .send()
                    .await
                {
                    Ok(_) => {
                        println!("  ✓ {}/{}", bucket, filename);
                        success += 1;
                    }
                    Err(e) => {
                        eprintln!("  ✗ {}/{}: {}", bucket, filename, e);
                        fail += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("  ✗ read {}: {}", path.display(), e);
                fail += 1;
            }
        }
    }

    (success, fail)
}

/// Chat uploads are stored as /data/chat_uploads/{conv_id}/{filename}
async fn migrate_chat_dir(client: &Client, base_dir: &str) -> (u64, u64) {
    let mut success = 0u64;
    let mut fail = 0u64;

    let mut conv_entries = match tokio::fs::read_dir(base_dir).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Cannot read {}: {}", base_dir, e);
            return (0, 0);
        }
    };

    while let Ok(Some(conv_entry)) = conv_entries.next_entry().await {
        let conv_path = conv_entry.path();
        if !conv_path.is_dir() { continue; }

        let conv_id = match conv_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let mut file_entries = match tokio::fs::read_dir(&conv_path).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Ok(Some(file_entry)) = file_entries.next_entry().await {
            let file_path = file_entry.path();
            if !file_path.is_file() { continue; }

            let filename = match file_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let key = format!("{}/{}", conv_id, filename);
            let mime = mime_from_ext(&filename);

            match tokio::fs::read(&file_path).await {
                Ok(bytes) => {
                    match client
                        .put_object()
                        .bucket("chat")
                        .key(&key)
                        .body(ByteStream::from(bytes))
                        .content_type(mime)
                        .send()
                        .await
                    {
                        Ok(_) => {
                            println!("  ✓ chat/{}", key);
                            success += 1;
                        }
                        Err(e) => {
                            eprintln!("  ✗ chat/{}: {}", key, e);
                            fail += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  ✗ read {}: {}", file_path.display(), e);
                    fail += 1;
                }
            }
        }
    }

    (success, fail)
}

/// Patch news_articles.cover_image_url from /api/public/news/images/X to CDN URL
fn patch_news_urls(db_url: &str, public_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use diesel::sql_types::Text;

    let mut conn = PgConnection::establish(db_url)?;

    let old_prefix = "/api/public/news/images/";
    let new_prefix = format!("{}/news/", public_url.trim_end_matches('/'));
    let like_pattern = format!("{}%", old_prefix);

    let rows = diesel::sql_query(
        "UPDATE news_articles SET cover_image_url = REPLACE(cover_image_url, $1, $2) WHERE cover_image_url LIKE $3",
    )
    .bind::<Text, _>(old_prefix)
    .bind::<Text, _>(&new_prefix)
    .bind::<Text, _>(&like_pattern)
    .execute(&mut conn)?;

    println!("Updated {} news article cover_image_url values", rows);
    Ok(())
}

fn mime_from_ext(filename: &str) -> &'static str {
    match filename.rsplit('.').next().map(|s| s.to_lowercase()).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}
