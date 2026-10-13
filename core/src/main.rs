use anyhow::Result;
use boosty_downloader_core::{
    ConsoleLogger, handle_menu, init_client, make_client, print_error, print_help, set_logger,
};

#[tokio::main]
async fn main() {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "-h" || arg == "--help")
    {
        print_help();
        return;
    }

    set_logger(ConsoleLogger);

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
