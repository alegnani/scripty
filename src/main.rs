use helper::*;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::{async_trait, Client};
use tracing::{info, warn};
use std::env;

use crate::languages::{run_pipeline, LanguagePool};

mod helper;
mod languages;

#[tokio::main]
async fn main() {
    let collector = tracing_subscriber::fmt().init();
    let _ = LANG_POOL.set(LanguagePool::new().await);
    let languages = LANG_POOL.get().unwrap().get_supported().await;

    info!("Language pool set");
    info!(?languages, "Supported");
    //const HELP_COMMAND: &str = "~help";
    //const HELP_MESSAGE: &str = "help message for scripty";

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Error whilst creating client");
    println!("Client created. Starting it");

    if let Err(e) = client.start().await {
        println!("Client error: {:?}", &e);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        info!("New message detected: {}", msg.content);
        if msg.content.starts_with("~run") {
            let reply = match run_pipeline(&msg.content).await {
                Ok(s) => format!("```{}\n```\nElapsed time: {}ms", s.output, s.execution_time.as_millis()),
                Err(e) => e.to_string(),
            };
            msg.reply(ctx.http, reply).await.unwrap();

        }
    }
    // TODO: fix duplicated code
    async fn message_update(&self, ctx: Context, new_data: serenity::model::event::MessageUpdateEvent) {
        let content = new_data.content.unwrap().clone();
        info!("A message has been edited: {}", content);
        if content.starts_with("~run") {
            let reply = match run_pipeline(&content).await {
                Ok(s) => format!("```{}\n```\nElapsed time: {}ms", s.output, s.execution_time.as_millis()),
                Err(e) => e.to_string(),
            };
            new_data.channel_id.say(ctx.http, reply).await.unwrap();
        }
    }

    async fn ready(&self, _ctx: Context, _data: serenity::model::prelude::Ready) {
        info!("Scripty is online and ready");
    }
}
