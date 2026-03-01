use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    types::{
        BucketLifecycleConfiguration, CompletedMultipartUpload, CompletedPart,
        ExpirationStatus, LifecycleExpiration, LifecycleRule, LifecycleRuleFilter,
    },
    Client,
};
use std::time::Duration;
use tracing::{error, info};

use super::types::{Bucket, FileInfo};

const MULTIPART_THRESHOLD: usize = 5 * 1024 * 1024; // 5MB

#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    presign_client: Client,
    public_url: String,
}

impl S3Storage {
    pub async fn new(endpoint: &str, access_key: &str, secret_key: &str, public_url: &str) -> Self {
        let make_client = |url: &str| -> Client {
            let creds = Credentials::new(access_key, secret_key, None, None, "minio");
            let config = aws_sdk_s3::Config::builder()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new("us-east-1"))
                .endpoint_url(url)
                .credentials_provider(creds)
                .force_path_style(true)
                .build();
            Client::from_conf(config)
        };

        // Use public_url for presigned URL generation so URLs are browser-accessible
        let presign_endpoint = if !public_url.is_empty() { public_url } else { endpoint };
        Self {
            client: make_client(endpoint),
            presign_client: make_client(presign_endpoint),
            public_url: public_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create all buckets, set policies and lifecycle rules
    pub async fn init_buckets(&self) {
        for bucket in Bucket::all() {
            let name = bucket.as_str();
            match self.client.head_bucket().bucket(name).send().await {
                Ok(_) => info!("Bucket '{}' already exists", name),
                Err(_) => match self.client.create_bucket().bucket(name).send().await {
                    Ok(_) => info!("Created bucket '{}'", name),
                    Err(e) => {
                        error!("Failed to create bucket '{}': {}", name, e);
                        continue;
                    }
                },
            }

            // Only Public bucket gets public-read policy; others are private by default
            if matches!(bucket, Bucket::Public) {
                let policy = serde_json::json!({
                    "Version": "2012-10-17",
                    "Statement": [{
                        "Effect": "Allow",
                        "Principal": {"AWS": ["*"]},
                        "Action": ["s3:GetObject"],
                        "Resource": [format!("arn:aws:s3:::{}/*", name)]
                    }]
                });
                if let Err(e) = self.client
                    .put_bucket_policy()
                    .bucket(name)
                    .policy(policy.to_string())
                    .send()
                    .await
                {
                    error!("Failed to set policy on '{}': {}", name, e);
                }
            }

            // Set 90-day auto-expiry on chat bucket
            if matches!(bucket, Bucket::Chat) {
                let expiration = LifecycleExpiration::builder().days(90).build();
                let filter = LifecycleRuleFilter::builder().prefix("").build();
                if let Ok(rule) = LifecycleRule::builder()
                    .id("chat-90day-expiry")
                    .status(ExpirationStatus::Enabled)
                    .filter(filter)
                    .expiration(expiration)
                    .build()
                {
                    if let Ok(config) = BucketLifecycleConfiguration::builder()
                        .rules(rule)
                        .build()
                    {
                        if let Err(e) = self.client
                            .put_bucket_lifecycle_configuration()
                            .bucket(name)
                            .lifecycle_configuration(config)
                            .send()
                            .await
                        {
                            error!("Failed to set lifecycle on '{}': {}", name, e);
                        }
                    }
                }
            }
        }
    }

    pub async fn put_object(
        &self,
        bucket: Bucket,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        if bytes.len() > MULTIPART_THRESHOLD {
            self.put_multipart(bucket, key, bytes, content_type).await
        } else {
            self.client
                .put_object()
                .bucket(bucket.as_str())
                .key(key)
                .body(ByteStream::from(bytes))
                .content_type(content_type)
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {}", e))?;
            Ok(self.public_url(bucket, key))
        }
    }

    async fn put_multipart(
        &self,
        bucket: Bucket,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        let create = self.client
            .create_multipart_upload()
            .bucket(bucket.as_str())
            .key(key)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| format!("Create multipart failed: {}", e))?;

        let upload_id = create.upload_id().unwrap_or_default().to_string();
        let mut parts = Vec::new();

        for (i, chunk) in bytes.chunks(MULTIPART_THRESHOLD).enumerate() {
            let part_num = (i + 1) as i32;
            let part = self.client
                .upload_part()
                .bucket(bucket.as_str())
                .key(key)
                .upload_id(&upload_id)
                .part_number(part_num)
                .body(ByteStream::from(chunk.to_vec()))
                .send()
                .await
                .map_err(|e| format!("Upload part {} failed: {}", part_num, e))?;
            parts.push(
                CompletedPart::builder()
                    .part_number(part_num)
                    .e_tag(part.e_tag().unwrap_or_default())
                    .build(),
            );
        }

        let completed = CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(bucket.as_str())
            .key(key)
            .upload_id(&upload_id)
            .multipart_upload(completed)
            .send()
            .await
            .map_err(|e| format!("Complete multipart failed: {}", e))?;

        Ok(self.public_url(bucket, key))
    }

    pub async fn presigned_url(
        &self,
        bucket: Bucket,
        key: &str,
        expires_secs: u64,
    ) -> Result<String, String> {
        let config = PresigningConfig::expires_in(Duration::from_secs(expires_secs))
            .map_err(|e| format!("Presigning config error: {}", e))?;
        let req = self.presign_client
            .get_object()
            .bucket(bucket.as_str())
            .key(key)
            .presigned(config)
            .await
            .map_err(|e| format!("Presigning failed: {}", e))?;
        Ok(req.uri().to_string())
    }

    pub async fn delete_object(&self, bucket: Bucket, key: &str) -> Result<(), String> {
        self.client
            .delete_object()
            .bucket(bucket.as_str())
            .key(key)
            .send()
            .await
            .map_err(|e| format!("S3 delete failed: {}", e))?;
        Ok(())
    }

    pub async fn list_objects(
        &self,
        bucket: Bucket,
        prefix: Option<&str>,
        limit: Option<i32>,
    ) -> Result<Vec<FileInfo>, String> {
        let mut req = self.client.list_objects_v2().bucket(bucket.as_str());
        if let Some(p) = prefix {
            req = req.prefix(p);
        }
        if let Some(l) = limit {
            req = req.max_keys(l);
        }

        let resp = req.send().await.map_err(|e| format!("S3 list failed: {}", e))?;
        let items = resp.contents().iter().map(|obj| {
            let key = obj.key().unwrap_or_default().to_string();
            FileInfo {
                url: self.public_url(bucket, &key),
                key,
                size: obj.size().unwrap_or(0),
                last_modified: obj.last_modified().map(|t| t.to_string()),
            }
        }).collect();
        Ok(items)
    }

    pub fn public_url(&self, bucket: Bucket, key: &str) -> String {
        format!("{}/{}/{}", self.public_url, bucket.as_str(), key)
    }
}
