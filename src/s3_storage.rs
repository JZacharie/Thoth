use anyhow::Result;
use s3::Bucket;
use s3::creds::Credentials;

use crate::config::S3Config;

pub struct S3Storage {
    bucket: Box<Bucket>,
}

impl S3Storage {
    pub fn new(config: &S3Config) -> Result<Option<Self>> {
        if config.secret_key.is_empty() {
            tracing::warn!("MINIO_SECRET_KEY not set, S3 upload disabled");
            return Ok(None);
        }

        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )?;

        let bucket =
            Bucket::new(&config.bucket, config.region.parse()?, credentials)?.with_path_style();

        Ok(Some(Self { bucket }))
    }

    pub async fn upload_png(&self, data: &[u8], key: &str) -> Result<String> {
        let response = self
            .bucket
            .put_object_with_content_type(key, data, "image/png")
            .await?;

        let code = response.status_code();
        if !(200..300).contains(&code) {
            anyhow::bail!("S3 upload failed with status {code}");
        }

        let url = format!("{}/{}/{}", self.bucket.url(), self.bucket.name(), key);
        tracing::info!("S3 upload successful: {url}");
        Ok(url)
    }
}
