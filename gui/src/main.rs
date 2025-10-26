use anyhow::Result;
use boosty_api::api_client::ApiClient;
use boosty_downloader_core::{init_client, make_client};
use iced::widget::{button, column, text};
use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application("Boosty Downloader", App::update, App::view)
        .antialiasing(true)
        .run()
}

#[derive(Default)]
struct App {
    client: Option<ApiClient>,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    InitializeClient,
    ClientInitialized(Result<ApiClient, String>),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InitializeClient => {
                self.status = "Initializing client...".to_string();
                let future = async {
                    match make_client().await {
                        Ok(client) => match init_client(&client).await {
                            Ok(_) => Ok(client),
                            Err(e) => Err(format!("Failed to init client: {}", e)),
                        },
                        Err(e) => Err(format!("Failed to create client: {}", e)),
                    }
                };
                Task::perform(future, Message::ClientInitialized)
            }
            Message::ClientInitialized(result) => {
                match result {
                    Ok(client) => {
                        self.client = Some(client);
                        self.status = "Client initialized successfully".to_string();
                    }
                    Err(e) => {
                        self.status = format!("Error: {}", e);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text(&self.status),
            button("Initialize Client").on_press(Message::InitializeClient),
        ]
        .padding(20)
        .spacing(10)
        .into()
    }
}
