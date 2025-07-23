
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::builder::CreateMessage;
use serenity::prelude::*;
use tokio::time::{interval, sleep};
use rss::Channel;
use rss::validation::Validate;
use std::error::Error;
use std::time::Duration;
use chrono;

struct Handler {
    last_entry_id: Option<String>,
    channel_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Start background task once on ready
        let ctx = ctx.clone();
        let mut handler = self.clone();
        let mut interval = interval(Duration::from_secs(300));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                println!("Background RSS check triggered at {}", chrono::Utc::now());
                if let Err(e) = handler.check_rss(&ctx).await {
                    println!("Error checking RSS in background: {:?}", e);
                }
            }
        });
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!rss" {
            println!("Received !rss command from {} at {}", msg.author.name, chrono::Utc::now());
            let channel = ctx.http.get_channel(ChannelId::from(self.channel_id))
                .await
                .unwrap_or_else(|e| {
                    println!("Failed to get channel: {:?}", e);
                    panic!("Channel access failed");
                })
                .guild()
                .unwrap_or_else(|| panic!("Channel is not in a guild"));

            if let Err(e) = channel
                .send_message(&ctx.http, CreateMessage::new().content("Checking RSS feed..."))
                .await
            {
                println!("Error sending message: {:?}", e);
            } else {
                sleep(Duration::from_secs(1)).await;
            }
        } else {
            println!("Ignored message: {} at {}", msg.content, chrono::Utc::now());
        }
    }
}

impl Handler {
    async fn check_rss(&mut self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        let content = reqwest::get("https://status.robertsspaceindustries.com/index.xml")
            .await?
            .bytes()
            .await?;

        let channel = Channel::read_from(&content[..])?;

        channel.validate()?;

        let latest_entry = channel.items().first();
        
        if let Some(entry) = latest_entry {
            let entry_id = entry.guid().map(|g| g.value().to_string())
                .unwrap_or_else(|| {
                    println!("Warning: No GUID, using default ID at {}", chrono::Utc::now());
                    format!("default-{}", chrono::Utc::now().timestamp())
                });

            if self.last_entry_id.as_ref().map_or(true, |id| id != &entry_id) {
                let pub_date = entry.pub_date().unwrap_or("No publication date");
                let title = entry.title().unwrap_or("No title");
                let description = entry.description().unwrap_or("No description")
                    .replace("<p>", "") // Remove HTML paragraph tags
                    .replace("</p>", "\n") // Convert to newlines
                    .replace("<!-- raw HTML omitted -->", ""); // Remove HTML comments
                let link = entry.link().unwrap_or("No link");

                let message = format!(
                    "**{}**\n*Published: {}*\n{}\n[{}]({})",
                    title, pub_date, description, title, link
                );

                let channel = ctx.http.get_channel(ChannelId::from(self.channel_id))
                    .await?
                    .guild()
                    .unwrap_or_else(|| panic!("Channel not in guild"));

                let _ = channel
                    .send_message(&ctx.http, CreateMessage::new().content(&message))
                    .await?;

                self.last_entry_id = Some(entry_id);
            } else {
                println!("No new RSS entry at {}", chrono::Utc::now());
            }
        } else {
            println!("No entries in RSS feed at {}", chrono::Utc::now());
        }
        Ok(())
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Handler {
            last_entry_id: self.last_entry_id.clone(),
            channel_id: self.channel_id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting bot at {}", chrono::Utc::now());
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let secrets_path = home_dir.join(".secrets");
    dotenv::from_path(&secrets_path).map_err(|e| format!("Failed to load .secrets: {}", e))?;

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in .secrets");
    let channel_id_str = std::env::var("CHANNEL_ID").expect("Expected CHANNEL_ID in .secrets");
    let channel_id: u64 = channel_id_str
        .parse()
        .map_err(|e| format!("Failed to parse CHANNEL_ID as u64: {}", e))?;

    {
        let mut client = Client::builder(&token, GatewayIntents::GUILD_MESSAGES)
            .event_handler(Handler {
                last_entry_id: None,
                channel_id,
            })
            .await?;

        println!("Client built, starting at {}", chrono::Utc::now());
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }
    Ok(())
}
