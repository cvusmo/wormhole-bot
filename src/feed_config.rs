// ~/src/feed_config.rs

use std::collections::HashMap;
use std::error::Error;

pub fn load_feeds() -> Result<HashMap<String, u64>, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let secrets_path = home_dir.join(".secrets");
    dotenv::from_path(&secrets_path).map_err(|e| format!("Failed to load .secrets: {e}"))?;

    let mut feeds = HashMap::new();

    // Load first channel and feed
    let channel_id_1: u64 = std::env::var("CHANNEL_ID_1")
        .expect("Expected CHANNEL_ID_1 in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID_1: {e}"))?;
    feeds.insert(
        "https://status.robertsspaceindustries.com/index.xml".to_string(),
        channel_id_1,
    );

    // Load second channel and feed
    let channel_id_2: u64 = std::env::var("CHANNEL_ID_2")
        .expect("Expected CHANNEL_ID_2 in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID_2: {e}"))?;
    feeds.insert(
        "https://archlinux.org/feeds/packages/all/core/".to_string(),
        channel_id_2,
    );

    Ok(feeds)
}
