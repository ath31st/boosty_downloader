use crate::post_handler;
use crate::{cli, parser};
use anyhow::{Context, Result};
use boosty_downloader_api::api_client::ApiClient;

pub async fn handle_menu(client: &mut ApiClient, posts_limit: i32) -> Result<bool> {
    cli::show_menu();
    let selected_menu = cli::read_input_menu();

    match selected_menu {
        1 => {
            let input = cli::read_user_input(cli::ENTER_PATH);
            if let Err(e) = process_boosty_url(client, posts_limit, &input).await {
                cli::print_error(&e)
            };
        }
        2 => {
            let entered_token = cli::read_user_input(cli::ENTER_ACCESS_TOKEN);
            client.set_bearer_token(&entered_token)?;
            cli::show_api_client_headers(&client.headers_as_map());
        }
        3 => {
            let entered_token = cli::read_user_input(cli::ENTER_REFRESH_TOKEN);
            let entered_device_id = cli::read_user_input(cli::ENTER_CLIENT_ID);
            client.set_refresh_token_and_device_id(&entered_token, &entered_device_id)?;
        }
        4 => {
            cli::exit_message();
            return Ok(false);
        }
        _ => cli::show_menu(),
    }
    Ok(true)
}

async fn process_boosty_url(client: &mut ApiClient, posts_limit: i32, input: &str) -> Result<()> {
    let parsed = parser::parse_boosty_url(input)
        .with_context(|| format!("Failed to parse Boosty URL '{}'", input))?;

    let result = match parsed {
        parser::BoostyUrl::Blog(blog) => {
            let multiple = client
                .fetch_posts(&blog, posts_limit)
                .await
                .with_context(|| format!("Failed to fetch posts for blog '{}'", blog))?;
            post_handler::PostsResult::Multiple(multiple)
        }
        parser::BoostyUrl::Post { blog, post_id } => {
            let single = client.fetch_post(&blog, &post_id).await.with_context(|| {
                format!("Failed to fetch post '{}' for blog '{}'", post_id, blog)
            })?;
            post_handler::PostsResult::Single(single)
        }
    };

    post_handler::post_processor(result)
        .await
        .with_context(|| format!("Error while processing post content: {}", input))?;

    Ok(())
}
