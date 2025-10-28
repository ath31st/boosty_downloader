use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Theme};

use crate::app::Screen;
use crate::messages::Message;

pub fn header_view<'a>(
    app_name: &'a str,
    status: &'a str,
    current_screen: &'a Screen,
) -> Element<'a, Message> {
    fn nav_button<'a>(
        label: &'static str,
        active: bool,
        msg: Message,
    ) -> button::Button<'a, Message> {
        button(label)
            .on_press(msg)
            .style(move |theme: &Theme, status| {
                let palette = theme.extended_palette();

                match status {
                    button::Status::Active => {
                        if active {
                            iced::widget::button::Style::default()
                                .with_background(palette.primary.strong.color)
                        } else {
                            iced::widget::button::Style::default()
                        }
                    }
                    _ => iced::widget::button::primary(theme, status),
                }
            })
    }

    let title_row = container(text(app_name).size(24)).center_x(Length::Fill);

    let nav_row = row![
        row![
            nav_button(
                "Main",
                *current_screen == Screen::Main,
                Message::SwitchToMain
            ),
            nav_button(
                "Config",
                *current_screen == Screen::Config,
                Message::SwitchToConfig
            ),
        ]
        .spacing(10)
        .width(Length::Shrink),
        text(status).size(16).width(Length::Shrink)
    ]
    .width(Length::Fill)
    .spacing(10);

    column![title_row, nav_row].spacing(10).padding(10).into()
}
