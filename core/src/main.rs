use anyhow::Result;
use boosty_downloader_core::{handle_menu, init_client, make_client, print_error};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let client = make_client().await?;
    init_client(&client).await?;

    loop {
        if !handle_menu(&client).await? {
            break;
        }
    }

    Ok(())
}
