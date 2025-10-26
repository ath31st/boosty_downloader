use iced::{
    Element,
    widget::{column, text, text_input},
};

use crate::{config_input_handler::ConfigInput, messages::Message};

pub fn config_screen_view(config: &ConfigInput) -> Element<'_, Message> {
    column![
        text("Posts Limit:"),
        text_input("Posts Limit", &config.posts_limit).on_input(Message::ConfigPostsLimitChanged),
        text("Access Token:"),
        text_input("Access Token", &config.access_token)
            .on_input(Message::ConfigAccessTokenChanged),
        text("Refresh Token:"),
        text_input("Refresh Token", &config.refresh_token)
            .on_input(Message::ConfigRefreshTokenChanged),
        text("Device ID:"),
        text_input("Device ID", &config.device_id).on_input(Message::ConfigDeviceIdChanged),
        text("Comments Reply Limit:"),
        text_input("Reply Limit", &config.reply_limit).on_input(Message::ConfigReplyLimitChanged),
        text("Comments Limit:"),
        text_input("Limit", &config.limit).on_input(Message::ConfigLimitChanged),
        text("Comments Order (top/bottom):"),
        text_input("Order", &config.order).on_input(Message::ConfigOrderChanged),
    ]
    .spacing(5)
    .into()
}
