// ~/src/feed_config.rs

use std::collections::HashMap;
use std::error::Error;

pub fn load_feeds() -> Result<HashMap<String, u64>, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let secrets_path = home_dir.join(".secrets");
    dotenv::from_path(&secrets_path).map_err(|e| format!("Failed to load .secrets: {e}"))?;

    let mut feeds = HashMap::new();

    // STAR CITIZEN RSI UPDATES
    let channel_id_1: u64 = std::env::var("CHANNEL_ID_1")
        .expect("Expected CHANNEL_ID_1 in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID_1: {e}"))?;
    feeds.insert(
        "https://status.robertsspaceindustries.com/index.xml".to_string(),
        channel_id_1,
    );

    // ARCH LINUX PACKAGE UPDATES
    let channel_id_2: u64 = std::env::var("CHANNEL_ID_2")
        .expect("Expected CHANNEL_ID_2 in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID_2: {e}"))?;
    feeds.insert(
        "https://archlinux.org/feeds/packages/".to_string(),
        channel_id_2,
    );

    // NASA JPL FEED
    let channel_id_3: u64 = std::env::var("CHANNEL_ID_3")
        .expect("Expected CHANNEL_ID_3 in .secrets")
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID_3: {e}"))?;
    feeds.insert(
        "https://www.nasa.gov/centers-and-facilities/jpl/feed/".to_string(),
        channel_id_3,
    );

    Ok(feeds)
}
