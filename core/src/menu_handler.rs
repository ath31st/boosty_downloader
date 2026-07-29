use crate::DownloadOptions;
use crate::cli;
use crate::comment_handler;
use crate::config;
use crate::config::AppConfig;
use crate::file_handler;
use crate::log_error;
use crate::log_info;
use crate::parser::BoostyUrl;
use crate::post_handler;
use crate::progress_reporter;
use crate::url_context;
use anyhow::{Context, Result, anyhow};
use boosty_api::api_client::ApiClient;
use boosty_api::traits::HasTitle;
use boosty_api::traits::IsAvailable;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn handle_menu(client: &ApiClient) -> Result<bool> {
    let selected_menu = cli::read_input_menu();

    match selected_menu {
        0 => {
            let cfg = config::load_config().await?;

            if let Some((input, offset_input)) = cli::read_download_url_and_offset()
                && let Some(download_options) = cli::read_download_options()
            {
                let offset_opt = if offset_input.is_empty() {
                    None
                } else {
                    Some(offset_input.as_str())
                };

                let ctx = url_context::build_url_context(&input, offset_opt)?;

                if let Err(e) =
                    process_boosty_url(
                        client,
                        &cfg,
                        &ctx.url,
                        ctx.offset,
                        download_options,
                        &CancellationToken::new(),
                    )
                    .await
                {
                    log_error!("{:#}", e);
                };
            }
        }
        1 => {
            let cfg = config::load_config().await?;

            if let Some(file_path_str) = cli::read_batch_file_path()
                && let Some(download_options) = cli::read_download_options()
                && let Err(e) =
                    process_batch_file(client, &cfg, &file_path_str, download_options).await
            {
                log_error!("Batch process failed: {:#}", e);
            }
        }
        2 => {
            if let Some(entered_token) = cli::read_access_token() {
                client.set_bearer_token(&entered_token).await?;
                config::update_config(|cfg| {
                    cfg.access_token = entered_token;
                    cfg.refresh_token = String::new();
                    cfg.device_id = String::new();
                })
                .await
                .with_context(|| "Failed to update config")?;
            }
        }
        3 => {
            if let Some((entered_token, entered_device_id)) = cli::read_refresh_and_client_id() {
                client
                    .set_refresh_token_and_device_id(&entered_token, &entered_device_id)
                    .await?;
                let pair = client.refresh_tokens().await?;
                cli::access_token_set(&pair.access_token);
                cli::refresh_token_set(&pair.refresh_token);
                cli::client_id_set(&entered_device_id);
                config::update_config(|cfg| {
                    cfg.access_token = pair.access_token;
                    cfg.refresh_token = pair.refresh_token;
                    cfg.device_id = entered_device_id;
                })
                .await
                .with_context(|| "Failed to update config")?;
            }
        }
        4 => {
            let mut cfg = config::load_config().await?;
            config::clear_auth(client, &mut cfg).await?;
            config::save_config(&cfg)
                .await
                .with_context(|| "Failed to clear tokens")?;
            cli::tokens_and_client_id_cleared();
        }
        5 => {
            let cfg = config::load_config().await?;

            if let Some(limit) = cli::read_posts_limit(cfg.posts_limit) {
                config::update_config(|cfg| cfg.posts_limit = limit)
                    .await
                    .with_context(|| "Failed to update posts limit")?;
            }
        }
        6 => {
            let cfg = config::load_config().await?;

            if let Some(new_path_opt) = cli::read_download_path(cfg.download_path.as_deref()) {
                config::update_config(|cfg| cfg.download_path = new_path_opt)
                    .await
                    .with_context(|| "Failed to update download path")?;
            }
        }
        7 => {
            let cfg = config::load_config().await?;

            if let Some(enable_comments) = cli::read_comments_status(cfg.comments.enabled) {
                config::update_config(|cfg| {
                    cfg.comments.enabled = enable_comments;
                })
                .await
                .with_context(|| "Failed to update comments status")?;

                let status = if enable_comments {
                    "enabled"
                } else {
                    "disabled"
                };
                cli::comments_toggled(status);
            }
        }
        8 => cli::show_api_client_headers(&client.headers_as_map()),
        9 => cli::show_config(&config::load_config().await?),
        10 => {
            cli::exit_message();
            return Ok(false);
        }
        _ => {}
    }
    Ok(true)
}

pub async fn process_boosty_url(
    client: &ApiClient,
    cfg: &AppConfig,
    url: &BoostyUrl,
    offset_url: Option<BoostyUrl>,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;
    let offset: Option<String> = match offset_url {
        Some(BoostyUrl::Post { blog, post_id }) => {
            crate::ensure_not_cancelled(cancel_token)?;
            let offset_post = client.get_post(&blog, &post_id).await?;
            Some(format!("{}:{}", offset_post.sort_order, offset_post.int_id))
        }
        _ => None,
    };

    let result = match &url {
        BoostyUrl::Blog(blog) => {
            crate::ensure_not_cancelled(cancel_token)?;
            let multiple = client
                .get_posts(blog, cfg.posts_limit, None, offset)
                .await
                .map_err(|e| anyhow!("Failed to fetch posts for blog '{blog}', {}", e))?;
            post_handler::PostsResult::Multiple(multiple)
        }
        BoostyUrl::Post { blog, post_id } => {
            crate::ensure_not_cancelled(cancel_token)?;
            let single = client.get_post(blog, post_id).await.map_err(|e| {
                anyhow!("Failed to fetch post '{post_id}' for blog '{blog}', {}", e)
            })?;
            post_handler::PostsResult::Single(Box::from(single))
        }
    };

    // Collect metadata for comments before posts are consumed by process_posts.
    let comment_targets: Vec<(String, String, String, i64)> = if cfg.comments.enabled {
        match &result {
            post_handler::PostsResult::Single(post) => {
                if post.not_available() {
                    Vec::new()
                } else {
                    vec![(
                        post.user.blog_url.clone(),
                        post.id.clone(),
                        post.safe_title(),
                        post.created_at,
                    )]
                }
            }
            post_handler::PostsResult::Multiple(posts) => posts
                .iter()
                .filter(|p| !p.not_available())
                .map(|p| {
                    (
                        p.user.blog_url.clone(),
                        p.id.clone(),
                        p.safe_title(),
                        p.created_at,
                    )
                })
                .collect(),
        }
    } else {
        Vec::new()
    };

    let download_path = &config::get_download_path(cfg);
    let post_files = post_handler::count_downloadable_files(&result, &download_options);
    let _progress = progress_reporter::SessionGuard::new(post_files);

    post_handler::process_posts(result, download_path, download_options.clone(), cancel_token)
        .await
        .with_context(|| {
            format!(
                "Error while processing post content: {}",
                match &url {
                    BoostyUrl::Blog(blog) => blog,
                    BoostyUrl::Post { blog, .. } => blog,
                }
            )
        })?;

    if !comment_targets.is_empty() {
        let mut comments_results = Vec::new();

        for (blog_url, post_id, safe_post_title, created_at) in comment_targets {
            crate::ensure_not_cancelled(cancel_token)?;
            match client
                .get_all_comments(
                    &blog_url,
                    &post_id,
                    cfg.comments.limit,
                    cfg.comments.reply_limit,
                    cfg.comments.order.as_deref(),
                )
                .await
            {
                Ok(comments) => {
                    comments_results.push(comment_handler::CommentsResult {
                        comments,
                        safe_post_title,
                        created_at,
                        blog_url,
                    });
                }
                Err(e) => {
                    log_error!("Failed to fetch comments for post '{post_id}': {e:#}");
                }
            }
        }

        if let Err(e) = comment_handler::process_comments(
            comments_results,
            download_path,
            download_options,
            cancel_token,
        )
        .await
        {
            if crate::is_cancelled_error(&e) {
                return Err(e);
            }
            log_error!("Error while processing comments: {e:#}");
        }
    }

    Ok(())
}

async fn process_batch_file(
    client: &ApiClient,
    cfg: &AppConfig,
    file_path_str: &str,
    download_options: DownloadOptions,
) -> Result<()> {
    let file_path = Path::new(file_path_str);
    let links = file_handler::read_links_from_file(file_path).await?;

    log_info!("Starting batch processing of {} links...", links.len());

    for link in links {
        log_info!("Processing: {link}");

        match url_context::build_url_context(&link, None) {
            Ok(ctx) => {
                if let Err(e) =
                    process_boosty_url(
                        client,
                        cfg,
                        &ctx.url,
                        ctx.offset,
                        download_options.clone(),
                        &CancellationToken::new(),
                    )
                    .await
                {
                    log_error!("Error processing link '{}': {e}", link);
                }
            }
            Err(e) => {
                log_error!("Invalid link format '{}': {e}", link);
            }
        }
    }

    log_info!("Batch processing finished.");
    Ok(())
}
