// ~/src/handler.rs 

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::Duration;

pub struct Handler {
    pub last_entry_ids: HashMap<String, String>,
    pub feeds: HashMap<String, u64>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log_info(&format("{} is connected!", ready.user.name));

        let ctx = ctx.clone();
        let mut handler = self.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::feed::interval_feed(handler.feeds, ctx, &mut handler.last_entry_ids).await {
                log_error(&format!("Error checking RSS: {:?}", e));
            }
        });
    }
    
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!rss" {
            log_info(&format!("Receieved !rss command from {}", msg.author.name));
            let channel_id = ChannelId::from(self.feeds.values().next().unwrap_or(&0));

            if let Ok(channel) = ctx.http.get_channel(channel_id).await {
                if let Some(guild_channel) = channel.guild() {
                    let _ = guild_channel
                        .send_message(&ctx.http, CreateMessage::new().content("Checking RSS feed..."))
                        .await;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Handler {
            last_entry_ids: self.last_entry_ids.clone(),
            feeds: self.feeds.clone(),
        }
    }
}
