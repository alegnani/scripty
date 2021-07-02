use helper::*;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::{
    async_trait,
    Client,
};
use std::env;

use crate::language_pool::LanguagePool;
use crate::tasks::pipeline;

mod helper;
mod language;
mod language_pool;
mod tasks;

#[tokio::main]
async fn main() {
    let _ = LANG_POOL.set(LanguagePool::new().await);

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
        println!("New message in channel");
        if !&msg.content.starts_with("~run") {
            return;
        }
        let reply = match pipeline(&msg.content).await {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        let reply = format!("```{}```", reply);
        msg.channel_id.say(ctx.http, reply).await.unwrap();
    }

    async fn ready(&self, _ctx: Context, data: serenity::model::prelude::Ready) {
        println!("{} is connected!", data.user.name);
    }
}
