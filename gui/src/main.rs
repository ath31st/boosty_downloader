use crate::app::App;

mod app;
mod config_input_handler;
mod messages;
mod views;

fn main() -> iced::Result {
    iced::application("Boosty Downloader", App::update, App::view)
        .theme(App::theme)
        .antialiasing(true)
        .run_with(App::initialize)
}
