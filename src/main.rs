use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::Result;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use discord_tui::infrastructure::{AppConfig, DiscordAuthClient, KeyringTokenStorage};
use discord_tui::presentation::App;

fn init_logging(config: &AppConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.to_string()));

    if let Some(log_path) = config.effective_log_path() {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let file_layer = fmt::layer()
            .with_writer(file)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .init();

        info!(path = %log_path.display(), "Logging initialized");
    } else {
        tracing_subscriber::registry().with(filter).init();
    }

    Ok(())
}

fn create_app() -> Result<(App, Option<String>)> {
    let config = AppConfig::parse();
    let cli_token = config.token.clone();

    init_logging(&config)?;

    info!(version = discord_tui::VERSION, "Starting Discordo");

    let auth_client = Arc::new(DiscordAuthClient::new()?);
    let token_storage = Arc::new(KeyringTokenStorage::new());

    let app = App::new(auth_client, token_storage);

    Ok((app, cli_token))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let (app, cli_token) = create_app()?;

    let mut terminal = ratatui::init();

    let result = app.run(&mut terminal, cli_token).await;

    ratatui::restore();

    result
}
