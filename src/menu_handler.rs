use crate::config;
use crate::post_handler;
use crate::{cli, parser};
use anyhow::{Context, Result};
use boosty_api::api_client::ApiClient;

pub async fn handle_menu(client: &ApiClient) -> Result<bool> {
    cli::show_menu();
    let selected_menu = cli::read_input_menu();

    match selected_menu {
        1 => {
            let cfg = config::load_config().await?;
            let input = cli::read_user_input(cli::ENTER_PATH);
            if let Err(e) = process_boosty_url(client, cfg.posts_limit, &input).await {
                cli::print_error(&e)
            };
        }
        2 => {
            let entered_token = cli::read_user_input(cli::ENTER_ACCESS_TOKEN);
            client.set_bearer_token(&entered_token).await?;
            config::update_config(|cfg| {
                cfg.access_token = entered_token;
                cfg.refresh_token = String::new();
                cfg.device_id = String::new();
            })
            .await
            .with_context(|| "Failed to update config")?;
        }
        3 => {
            let entered_token = cli::read_user_input(cli::ENTER_REFRESH_TOKEN);
            let entered_device_id = cli::read_user_input(cli::ENTER_CLIENT_ID);
            client
                .set_refresh_token_and_device_id(&entered_token, &entered_device_id)
                .await?;
            config::update_config(|cfg| {
                cfg.access_token = String::new();
                cfg.refresh_token = entered_token;
                cfg.device_id = entered_device_id;
            })
            .await
            .with_context(|| "Failed to update config")?;
        }
        4 => {
            client.clear_refresh_and_device_id().await;
            config::update_config(|cfg| {
                cfg.access_token = String::new();
                cfg.refresh_token = String::new();
                cfg.device_id = String::new();
            })
            .await
            .with_context(|| "Failed to clear tokens")?;
            cli::tokens_and_client_id_cleared();
        }
        5 => {
            let cfg = config::load_config().await?;
            let prompt = format!("{} (current: {}):", cli::ENTER_POSTS_LIMIT, cfg.posts_limit);
            let entered_posts_limit = cli::read_user_input(&prompt);
            match entered_posts_limit
                .trim()
                .parse::<usize>()
                .map_err(anyhow::Error::from)
            {
                Ok(limit) => {
                    config::update_config(|cfg| cfg.posts_limit = limit)
                        .await
                        .with_context(|| "Failed to update posts limit")?;
                }
                Err(e) => {
                    cli::print_error(&e);
                }
            }
        }
        6 => cli::show_api_client_headers(&client.headers_as_map()),
        7 => {
            cli::exit_message();
            return Ok(false);
        }
        _ => cli::show_menu(),
    }
    Ok(true)
}

async fn process_boosty_url(client: &ApiClient, posts_limit: usize, input: &str) -> Result<()> {
    let parsed = parser::parse_boosty_url(input)
        .with_context(|| format!("Failed to parse Boosty URL '{input}'"))?;

    let result = match parsed {
        parser::BoostyUrl::Blog(blog) => {
            let multiple = client
                .get_posts(&blog, posts_limit)
                .await
                .with_context(|| format!("Failed to fetch posts for blog '{blog}'"))?;
            post_handler::PostsResult::Multiple(multiple.data)
        }
        parser::BoostyUrl::Post { blog, post_id } => {
            let single = client
                .get_post(&blog, &post_id)
                .await
                .with_context(|| format!("Failed to fetch post '{post_id}' for blog '{blog}'"))?;
            post_handler::PostsResult::Single(Box::from(single))
        }
    };

    post_handler::process_posts(result)
        .await
        .with_context(|| format!("Error while processing post content: {input}"))?;

    Ok(())
}
