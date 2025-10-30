use boosty_api::api_client::ApiClient;
use boosty_downloader_core::AppConfig;

#[derive(Debug, Default)]
pub struct AppState {
    pub client: Option<ApiClient>,
    pub config: AppConfig,
}
