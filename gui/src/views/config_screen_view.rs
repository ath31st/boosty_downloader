use iced::{
    Element,
    widget::{button, column, text, text_input},
};

use crate::{app::App, messages::Message};

pub fn config_screen_view(app: &'_ App) -> Element<'_, Message> {
    column![
        text("Posts Limit:"),
        text_input("Posts Limit", &app.config_posts_limit)
            .on_input(Message::ConfigPostsLimitChanged),
        text("Access Token:"),
        text_input("Access Token", &app.config_access_token)
            .on_input(Message::ConfigAccessTokenChanged),
        text("Refresh Token:"),
        text_input("Refresh Token", &app.config_refresh_token)
            .on_input(Message::ConfigRefreshTokenChanged),
        text("Device ID:"),
        text_input("Device ID", &app.config_device_id).on_input(Message::ConfigDeviceIdChanged),
        text("Comments Reply Limit:"),
        text_input("Reply Limit", &app.config_reply_limit)
            .on_input(Message::ConfigReplyLimitChanged),
        text("Comments Limit:"),
        text_input("Limit", &app.config_limit).on_input(Message::ConfigLimitChanged),
        text("Comments Order (top/bottom):"),
        text_input("Order", &app.config_order).on_input(Message::ConfigOrderChanged),
        button("Save Config").on_press(Message::SaveConfig),
    ]
    .spacing(5)
    .into()
}
