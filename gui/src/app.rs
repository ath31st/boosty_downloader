use boosty_api::api_client::ApiClient;
use boosty_downloader_core::config::AppConfig;
use iced::widget::{button, column, row, text};
use iced::{Element, Task};

use crate::messages::Message;
use crate::views::{config_screen_view, main_screen_view};

#[derive(Default)]
pub struct App {
    client: Option<ApiClient>,
    status: String,
    current_screen: Screen,
    pub(crate) url_input: String,
    config: AppConfig,
    pub(crate) config_posts_limit: String,
    pub(crate) config_access_token: String,
    pub(crate) config_refresh_token: String,
    pub(crate) config_device_id: String,
    pub(crate) config_reply_limit: String,
    pub(crate) config_limit: String,
    pub(crate) config_order: String,
}

#[derive(Default, PartialEq)]
enum Screen {
    #[default]
    Main,
    Config,
}

impl App {
    pub fn initialize() -> (Self, Task<Message>) {
        let app = Self {
            status: "Initializing...".to_string(),
            ..Default::default()
        };

        let init_task = Task::perform(
            async {
                let config_result = boosty_downloader_core::config::load_config()
                    .await
                    .map_err(|e| format!("Failed to load config: {e}"));

                let config = match config_result {
                    Ok(config) => config,
                    Err(e) => return Err(e),
                };

                let client_result = boosty_downloader_core::make_client()
                    .await
                    .map_err(|e| format!("Failed to create client: {e}"));

                let client = match client_result {
                    Ok(client) => client,
                    Err(e) => return Err(e),
                };

                boosty_downloader_core::init_client(&client)
                    .await
                    .map_err(|e| format!("Failed to init client: {e}"))?;

                Ok((client, config))
            },
            Message::ClientInitialized,
        );

        (app, init_task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ClientInitialized(result) => {
                match result {
                    Ok((client, config)) => {
                        self.client = Some(client);
                        self.config = config;
                        self.status = "Initialization successful".to_string();
                    }
                    Err(e) => {
                        self.status = e;
                    }
                }
                Task::none()
            }
            Message::SwitchToMain => {
                self.current_screen = Screen::Main;
                Task::none()
            }
            Message::SwitchToConfig => {
                self.current_screen = Screen::Config;
                self.config_posts_limit = self.config.posts_limit.to_string();
                self.config_access_token = self.config.access_token.clone();
                self.config_refresh_token = self.config.refresh_token.clone();
                self.config_device_id = self.config.device_id.clone();
                self.config_reply_limit = self
                    .config
                    .comments
                    .reply_limit
                    .map_or(String::new(), |v| v.to_string());
                self.config_limit = self
                    .config
                    .comments
                    .limit
                    .map_or(String::new(), |v| v.to_string());
                self.config_order = self.config.comments.order.clone().unwrap_or_default();
                Task::none()
            }
            Message::UrlInputChanged(value) => {
                self.url_input = value;
                Task::none()
            }
            Message::DownloadPressed => {
                // Заглушка: имитация скачивания
                self.status = format!("Downloading from URL: {} (stub)", self.url_input);
                Task::none()
            }
            Message::ConfigPostsLimitChanged(value) => {
                self.config_posts_limit = value;
                Task::none()
            }
            Message::ConfigAccessTokenChanged(value) => {
                self.config_access_token = value;
                Task::none()
            }
            Message::ConfigRefreshTokenChanged(value) => {
                self.config_refresh_token = value;
                Task::none()
            }
            Message::ConfigDeviceIdChanged(value) => {
                self.config_device_id = value;
                Task::none()
            }
            Message::ConfigReplyLimitChanged(value) => {
                self.config_reply_limit = value;
                Task::none()
            }
            Message::ConfigLimitChanged(value) => {
                self.config_limit = value;
                Task::none()
            }
            Message::ConfigOrderChanged(value) => {
                self.config_order = value;
                Task::none()
            }
            Message::SaveConfig => {
                if let Ok(limit) = self.config_posts_limit.parse() {
                    self.config.posts_limit = limit;
                }
                self.config.access_token = self.config_access_token.clone();
                self.config.refresh_token = self.config_refresh_token.clone();
                self.config.device_id = self.config_device_id.clone();
                self.config.comments.reply_limit = self.config_reply_limit.parse().ok();
                self.config.comments.limit = self.config_limit.parse().ok();
                self.config.comments.order =
                    Some(self.config_order.clone()).filter(|s| !s.is_empty());
                let config = self.config.clone();
                Task::perform(
                    async move {
                        boosty_downloader_core::config::save_config(&config)
                            .await
                            .map_err(|e| format!("Failed to save config: {e}"))
                    },
                    Message::ConfigSaved,
                )
            }
            Message::ConfigSaved(result) => {
                match result {
                    Ok(_) => self.status = "Config saved".to_string(),
                    Err(e) => self.status = format!("Failed to save config: {}", e),
                }
                Task::none()
            }
        }
    }

    pub fn view(&'_ self) -> Element<'_, Message> {
        let navigation = row![
            button("Main").on_press(Message::SwitchToMain),
            button("Config").on_press(Message::SwitchToConfig),
        ]
        .spacing(10);

        let content = match self.current_screen {
            Screen::Main => main_screen_view(self),
            Screen::Config => config_screen_view(self),
        };

        column![navigation, text(&self.status), content]
            .padding(20)
            .spacing(10)
            .into()
    }
}
