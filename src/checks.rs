use anyhow::Context;
use boosty_api::api_client::ApiClient;
use std::time::Duration;
use tokio::time::timeout;

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
