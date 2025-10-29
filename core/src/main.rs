use anyhow::Result;
use boosty_downloader_core::{
    console_logger::ConsoleLogger, handle_menu, init_client, logger, make_client, print_error,
};

#[tokio::main]
async fn main() {
    logger::set_logger(ConsoleLogger);

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
