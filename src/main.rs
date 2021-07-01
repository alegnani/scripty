use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::channel::Message,
    Client,
};
use std::{env, fmt::format, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

mod language;
mod language_pool;
mod script_service;

#[tokio::main]
async fn main() {
    const HELP_COMMAND: &str = "~help";
    const HELP_MESSAGE: &str = "help message for scripty";

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
        println!("new message: {}", msg.content);
        if msg.content == "~ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "Pong").await {
                println!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _ctx: Context, data: serenity::model::prelude::Ready) {
        println!("{} is connected!", data.user.name);
    }
}
