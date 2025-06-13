use crate::post_handler;
use crate::{cli, parser};
use anyhow::Result;
use roosty_downloader_api::api_client::ApiClient;

pub async fn handle_menu(client: &mut ApiClient) -> Result<bool> {
    cli::show_menu();
    let selected_menu = cli::read_input_menu();

    match selected_menu {
        1 => {
            let input = cli::read_user_input(cli::ENTER_PATH);

            if let Some(parsed) = parser::parse_boosty_url(&input) {
                let result = match parsed {
                    parser::BoostyUrl::Blog(blog) => {
                        let multiple = client.fetch_posts(&blog, 4).await?;
                        post_handler::PostsResult::Multiple(multiple)
                    }
                    parser::BoostyUrl::Post { blog, post_id } => {
                        let single = client.fetch_post(&blog, &post_id).await?;
                        post_handler::PostsResult::Single(single)
                    }
                };

                post_handler::post_processor(result).await?
            } else {
                cli::wrong_content_url(&input);
            }
        }
        2 => {
            let entered_token = cli::read_user_input(cli::ENTER_TOKEN);
            client.set_bearer_token(&entered_token);
            cli::show_api_client_headers(&client.headers_as_map());
        }
        3 => {
            cli::exit_message();
            return Ok(false);
        }
        _ => cli::show_menu(),
    }
    Ok(true)
}
