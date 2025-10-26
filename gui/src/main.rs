use crate::app::App;

mod app;
mod messages;
mod views;

fn main() -> iced::Result {
    iced::application("Boosty Downloader", App::update, App::view)
        .antialiasing(true)
        .run_with(App::initialize)
}
