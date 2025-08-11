use crate::logger;

use rss::Channel;
use rss::validation::{Validate, ValidationError};
use serenity::model::id::ChannelId;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::time::{interval, Duration};
use chrono::Utc;

// Error enum (made public)
#[derive(Debug)]
pub enum FeedError {
    Reqwest(reqwest::Error),
    Serenity(serenity::Error),
    Rss(rss::Error),
    Validation(ValidationError),
}

impl std::error::Error for FeedError {}
impl std::fmt::Display for FeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FeedError::Reqwest(e) => write!(f, "Reqwest error: {e}"),
            FeedError::Serenity(e) => write!(f, "Serenity error: {e}"),
            FeedError::Rss(e) => write!(f, "RSS error: {e}"),
            FeedError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl From<reqwest::Error> for FeedError {
    fn from(e: reqwest::Error) -> Self {
        FeedError::Reqwest(e)
    }
}

impl From<serenity::Error> for FeedError {
    fn from(e: serenity::Error) -> Self {
        FeedError::Serenity(e)
    }
}

impl From<rss::Error> for FeedError {
    fn from(e: rss::Error) -> Self {
        FeedError::Rss(e)
    }
}

impl From<ValidationError> for FeedError {
    fn from(e: ValidationError) -> Self {
        FeedError::Validation(e)
    }
}

// Remove unused FeedItem struct to avoid dead_code warning
// #[derive(Debug)]
// struct FeedItem {
//     title: String,
//     description: String,
//     link: String,
//     pub_date: String,
//     thumbnail: Option<String>,
// }

// Fetch and validate RSS Channel
pub async fn feed_fetch(url: &str) -> Result<Channel, FeedError> {
    logger::log_info(&format!("Fetching RSS from: {url}"));
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    channel.validate()?;
    Ok(channel)
}

// Format RSS feed
pub fn feed_format(item: &rss::Item) -> String {
    let pub_date = item.pub_date().unwrap_or("No publication date").to_string();
    let title = item.title().unwrap_or("No title").to_string();
    let description = item.description().unwrap_or("No description")
        .replace("<p>", "")
        .replace("</p>", "")
        .replace("<strong>", "")
        .replace("</strong>", "")
        .replace("<!-- raw HTML omitted -->", "")
        .chars()
        .take(2000)
        .collect::<String>();
    let link = item.link().unwrap_or("No link").to_string();
    format!("**{title}**\n*Published: {pub_date}*\n{description}\n[{title}]({link})")
}

// Interval updates for RSS feed with embeds
pub async fn feed_interval(
    feeds: HashMap<String, u64>,
    ctx: Context,
    last_entry_ids: &mut HashMap<String, String>,
) -> Result<(), FeedError> {
    let mut interval = interval(Duration::from_secs(600));

    loop {
        interval.tick().await;
        for (url, channel_id) in &feeds {
            let channel = feed_fetch(url).await?;
            if let Some(entry) = channel.items().first() {
                let entry_id = entry
                    .guid()
                    .map(|g| g.value().to_string())
                    .unwrap_or_else(|| format!("default-{}", Utc::now().timestamp()));
                if last_entry_ids.get(url).map_or(true, |id| id != &entry_id) {
                    // Parse thumbnail from enclosure
                    let thumbnail = entry
                        .enclosure()
                        .and_then(|enc| Some(enc.url().to_string()));

                    // Create embed
                    let mut embed = CreateEmbed::new()
                        .title(entry.title().unwrap_or("No title"))
                        .description({
                            let desc = entry.description().unwrap_or("No description")
                                .replace("<p>", "")
                                .replace("</p>", "")
                                .replace("<strong>", "")
                                .replace("</strong>", "")
                                .replace("<!-- raw HTML omitted -->", "")
                                .chars()
                                .take(2048)
                                .collect::<String>();
                            if desc.is_empty() { "No description available".to_string() } else { desc }
                        })
                        .url(entry.link().unwrap_or("No link"))
                        .timestamp(Utc::now())
                        .field("Published", entry.pub_date().unwrap_or("No publication date"), false);
                    if let Some(thumb_url) = thumbnail {
                        embed = embed.thumbnail(thumb_url);
                    }

                    // Send embed
                    let channel = ctx.http.get_channel(ChannelId::from(*channel_id)).await?;
                    let message = feed_format(entry);
                    if let Some(guild_channel) = channel.guild() {
                        guild_channel
                        .send_message(&ctx.http, CreateMessage::new().content(&message))
                        .await?;
                    }

                    last_entry_ids.insert(url.clone(), entry_id);
                }
            }
        }
    }
}
