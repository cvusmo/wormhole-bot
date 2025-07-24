// ~/src/handler.rs 

use crate::logger::{log_info, log_error};

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::time::Duration;

pub struct Handler {
    pub last_entry_ids: HashMap<String, String>,
    pub feeds: HashMap<String, u64>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log_info(&format!("{} is connected!", ready.user.name));

        let ctx = ctx.clone();
        let mut handler = self.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::feed::feed_interval(handler.feeds, ctx, &mut handler.last_entry_ids).await {
                log_error(&format!("Error checking RSS: {e:?}"));
            }
        });
    }
    
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!rss" {
            log_info(&format!("Receieved !rss command from {}", msg.author.name));
            let channel_id = self.feeds.values().next().copied().unwrap_or(0);

            if let Ok(channel) = ctx.http.get_channel(channel_id.into()).await {
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
