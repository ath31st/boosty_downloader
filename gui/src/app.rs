use boosty_api::api_client::ApiClient;
use boosty_downloader_core::AppConfig;
use iced::widget::column;
use iced::{Element, Task, Theme};

use crate::config_form::ConfigInput;
use crate::messages::Message;
use crate::views::{config_view, header_view, main_view};

#[derive(Default)]
pub struct App {
    client: Option<ApiClient>,
    status: String,
    current_screen: Screen,
    url_input: String,
    config: AppConfig,
    config_input: ConfigInput,
}

#[derive(Default, PartialEq)]
pub enum Screen {
    #[default]
    Main,
    Config,
}

impl App {
    pub fn initialize() -> (Self, Task<Message>) {
        let app = Self {
            status: "Initializing...".to_string(),
            config_input: ConfigInput::default(),
            ..Default::default()
        };

        let init_task = Task::perform(
            async {
                let config = boosty_downloader_core::load_config()
                    .await
                    .map_err(|e| format!("Failed to load config: {e}"))?;
                let client = boosty_downloader_core::make_client()
                    .await
                    .map_err(|e| format!("Failed to create client: {e}"))?;
                boosty_downloader_core::init_client(&client)
                    .await
                    .map_err(|e| format!("Failed to init client: {e}"))?;
                Ok((client, config))
            },
            |result| Message::ClientInitialized(Box::new(result)),
        );

        (app, init_task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ClientInitialized(result) => {
                match *result {
                    Ok((client, config)) => {
                        self.client = Some(client);
                        self.config = config.clone();
                        self.config_input = ConfigInput::from_config(&config);
                        self.status = "Initialization successful".to_string();
                    }
                    Err(e) => self.status = e,
                }
                Task::none()
            }
            Message::SwitchToConfig => {
                self.current_screen = Screen::Config;
                self.config_input = ConfigInput::from_config(&self.config);
                Task::none()
            }
            Message::UrlInputChanged(value) => {
                self.url_input = value;
                Task::none()
            }
            Message::DownloadPressed => {
                self.status = format!("Downloading from URL: {} (stub)", self.url_input);
                Task::none()
            }
            Message::ConfigPostsLimitChanged(value) => {
                self.config_input.posts_limit = value;
                self.save_config_task()
            }
            Message::ConfigAccessTokenChanged(value) => {
                self.config_input.access_token = value;
                self.save_config_task()
            }
            Message::ConfigRefreshTokenChanged(value) => {
                self.config_input.refresh_token = value;
                self.save_config_task()
            }
            Message::ConfigDeviceIdChanged(value) => {
                self.config_input.device_id = value;
                self.save_config_task()
            }
            Message::ConfigReplyLimitChanged(value) => {
                self.config_input.reply_limit = value;
                self.save_config_task()
            }
            Message::ConfigLimitChanged(value) => {
                self.config_input.limit = value;
                self.save_config_task()
            }
            Message::ConfigOrderChanged(value) => {
                self.config_input.order = value;
                self.save_config_task()
            }
            Message::ConfigSaved(result) => {
                match result {
                    Ok(_) => self.status = "Config saved".to_string(),
                    Err(e) => self.status = format!("Failed to save config: {e}"),
                }
                Task::none()
            }
            Message::SwitchToMain => {
                self.current_screen = Screen::Main;
                Task::none()
            }
        }
    }

    pub fn view(&'_ self) -> Element<'_, Message> {
        let content = match self.current_screen {
            Screen::Main => main_view(&self.url_input),
            Screen::Config => config_view(&self.config_input),
        };

        column![
            header_view("Boosty Downloader", &self.status, &self.current_screen,),
            content
        ]
        .padding(20)
        .spacing(10)
        .into()
    }

    pub fn theme(_state: &App) -> Theme {
        Theme::TokyoNight
    }

    fn save_config_task(&mut self) -> Task<Message> {
        let config = match self.config_input.to_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("Invalid config: {e}");
                return Task::none();
            }
        };

        self.config = config.clone();

        let client = match self.client.as_ref() {
            Some(c) => c.clone(),
            None => return Task::none(),
        };

        Task::perform(
            async move {
                boosty_downloader_core::apply_config(&client)
                    .await
                    .map_err(|e| format!("Failed to apply config: {e}"))?;
                boosty_downloader_core::save_config(&config)
                    .await
                    .map_err(|e| format!("Failed to save config: {e}"))
            },
            Message::ConfigSaved,
        )
    }
}
