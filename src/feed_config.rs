// ~/src/feed_config.rs

use std::collections::HashMap;
use std::error::Error;

pub fn load_feeds() -> Result<HashMap<String, u64>, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let secrets_path = home_dir.join(".secrets");
    dotenv::from_path(&secrets_path).map_err(|e| format!("Failed to load .secrets: {e}"))?;

    let channel_id: u64 = std::env::var("CHANNEL_ID")
        .expect("Expected CHANNEL_ID in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID: {e}"))?;

    let mut feeds = HashMap::new();
    feeds.insert(
        "https://status.robertsspaceindustries.com/index.xml".to_string(),
        channel_id,
    );

    Ok(feeds)
}
