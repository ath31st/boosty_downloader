use iced::{
    Element, Length,
    widget::{button, container, row, text_input},
};

use crate::messages::Message;

pub fn main_view(url_input: &str) -> Element<'_, Message> {
    let input = text_input("Enter URL", url_input).on_input(Message::UrlInputChanged);

    let download_button = button("Download").on_press(Message::DownloadPressed);

    container(
        row![input, download_button]
            .spacing(10)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
