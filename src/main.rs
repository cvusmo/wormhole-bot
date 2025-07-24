// ~/src/main.rs

use wormhole_rss::{feed_config, handler, logger};
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize log
    logger::init_logger();
    logger::log_info("Starting wormhole-rss...");

    // Load config from .secrets
    let discord_token = env::var("DICORD_TOKEN").expect("Expected DISCORD_TOKEN in .secrets");

    // Load feeds
    let feeds = feed_config::load_feeds()?;

    // Create Discord client with Handler
    let mut client = Client::builder(&discord_token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(handler::Handler {
            last_entry_ids: HashMap::new(),
            feeds,
        })
        .await?;

    logger::log_info("Client built, starting...");
    if let Err(e) = client.start().await {
        logger::log_error(&format!("Client error: {e:?}"));
    }

    Ok(())

}
