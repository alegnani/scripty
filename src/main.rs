use cli::parse_command;
use helper::*;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::{async_trait, Client};
use std::env;
use tracing::{error, info};

use crate::languages::LanguagePool;

mod cli;
mod helper;
mod languages;

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::never(".", "scripty.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    create_docker_executors().await.unwrap();

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

    if let Err(e) = client.start().await {
        error!("Client error: {}", &e);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let _typing = msg.channel_id.start_typing(&ctx.http).unwrap();
        let cmd = &msg.content;
        if let Some(reply) = parse_command(cmd).await {
            info!("New message is a scripty command!");
            msg.reply(ctx.http, reply).await.unwrap();
        }
    }
    async fn message_update(
        &self,
        ctx: Context,
        new_data: serenity::model::event::MessageUpdateEvent,
    ) {
        let _typing = new_data.channel_id.start_typing(&ctx.http).unwrap();
        let cmd = &new_data.content.unwrap();
        if let Some(reply) = parse_command(cmd).await {
            info!("Edited message is a scripty command!");
            new_data.channel_id.say(ctx.http, reply).await.unwrap();
        }
    }

    async fn ready(&self, _ctx: Context, _data: serenity::model::prelude::Ready) {
        info!("Scripty is online and ready");
    }
}
