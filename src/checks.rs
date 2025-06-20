use anyhow::Context;
use boosty_api::api_client::ApiClient;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

pub fn check_ffmpeg() -> anyhow::Result<()> {
    match Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        _ => Err(anyhow::anyhow!(
            "ffmpeg not found or not executable. Please install ffmpeg and ensure it is in PATH."
        )),
    }
}

pub async fn check_api(client: &ApiClient) -> anyhow::Result<()> {
    let fake_blog = "nonexistent";
    let fut = client.fetch_posts(fake_blog, 1);
    match timeout(Duration::from_secs(5), fut).await {
        Ok(Ok(_)) => {
            Ok(())
        }
        Ok(Err(e)) => {
            Err(e).with_context(|| "Failed to reach Boosty API")
        }
        Err(_) => Err(anyhow::anyhow!("Timeout when connecting to Boosty API")),
    }
}
