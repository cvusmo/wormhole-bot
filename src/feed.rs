// ~/src/feed.rs

use rss::Channel;
use rss::validation::Validate;
use serenity::model::id::ChannelId;
use serenity::builder::CreateMessage;
use serenity::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{interval, Duration};

pub async fn feed_fetch(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    channel.validate()?;
    Ok(channel)
}

pub fn feed_format(item: &rss::Item) -> String {
    let pub_date = item.pub_date().unwrap_or("No publication date");
    let title = item.title().unwrap_or("No title");
    let description = item.description().unwrap_or("No description")
        .replace("<p>", "")
        .replace("</p>", "")         
        .replace("<strong>", "") 
        .replace("</strong>", "")
        .replace("<!-- raw HTML omitted -->", "");

    let link = item.link().unwrap_or("No link");
    //format!("**{}**\n*Published: {}*\n{}\n[{}]({})", title, pub_date, description, title, link)
    format!("**{title}**\n*Published: {pub_date}*\n{description}\n[{title}]({link})")
}

pub async fn feed_interval(
    feeds: HashMap<String, u64>,
    ctx: Context,
    last_entry_ids: &mut HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let mut interval = interval(Duration::from_secs(300));
    
    loop {
        interval.tick().await;
        for (url, channel_id) in &feeds {
            let channel = feed_fetch(url).await?;
            if let Some(entry) = channel.items().first() {
                let entry_id = entry
                    .guid()
                    .map(|g| g.value().to_string())
                    .unwrap_or_else(|| format!("default-{}", chrono::Utc::now().timestamp()));
                //if last_entry_ids.get(url).map_or(true, |id| id !=&entry_id) {
                if last_entry_ids.get(url) != Some(&entry_id) {
                    let message = feed_format(entry);
                    let channel = ctx.http.get_channel(ChannelId::from(*channel_id)).await?;
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
    Ok(())
}
