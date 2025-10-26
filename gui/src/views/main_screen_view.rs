use iced::{
    Element,
    widget::{button, column, text_input},
};

use crate::{app::App, messages::Message};

pub fn main_screen_view(app: &'_ App) -> Element<'_, Message> {
    column![
        text_input("Enter URL", &app.url_input).on_input(Message::UrlInputChanged),
        button("Download").on_press(Message::DownloadPressed),
    ]
    .spacing(10)
    .into()
}
