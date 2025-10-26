use anyhow::Result;
use boosty_downloader_core::{cli, init_client, make_client, menu_handler};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        cli::print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let client = make_client().await?;
    init_client(&client).await?;

    loop {
        if !menu_handler::handle_menu(&client).await? {
            break;
        }
    }

    Ok(())
}
