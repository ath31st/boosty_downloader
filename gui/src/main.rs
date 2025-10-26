use iced::{Settings, Size};

use crate::app::App;

mod app;
mod config_form;
mod messages;
mod views;

fn main() -> iced::Result {
    let window = iced::window::Settings {
        size: Size::new(600.0, 700.0),
        resizable: false,
        ..Default::default()
    };
    let settings = Settings {
        antialiasing: true,
        ..Default::default()
    };

    iced::application("Boosty Downloader", App::update, App::view)
        .window(window)
        .theme(App::theme)
        .settings(settings)
        .run_with(App::initialize)
}
