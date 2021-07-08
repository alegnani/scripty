use helper::*;
use std::env;
use tracing::{error, info};

use crate::languages::LanguagePool;
use crate::cli::get_client;

mod cli;
mod helper;
mod languages;
mod replies;

#[tokio::main]
async fn main() {
    // let file_appender = tracing_appender::rolling::never(".", "scripty.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt().with_writer(non_blocking).init();
    tracing_subscriber::fmt().init();

    create_docker_executors().await.unwrap();

    let _ = LANG_POOL.set(LanguagePool::new().await);
    let languages = LANG_POOL.get().unwrap().get_supported().await;

    info!("Language pool set");
    info!(?languages, "Supported");
    //const HELP_COMMAND: &str = "~help";
    //const HELP_MESSAGE: &str = "help message for scripty";

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set");

    let mut client = get_client(&token).await;

    if let Err(e) = client.start().await {
        error!("Client error: {}", &e);
    }
}
