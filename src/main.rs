use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::gateway::ActivityData;
use serenity::model::{application::Interaction, prelude::*};
use songbird::SerenityInit;
use std::sync::Arc;
use tracing::{error, info};

mod commands;
mod config;
mod constants;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::Command(command) = interaction {
      commands::handle_commands(&ctx, command).await;
    }
  }

  async fn ready(&self, ctx: Context, ready: Ready) {
    let activity = ActivityData::playing("with 🍊");
    ctx.set_activity(Some(activity));

    commands::register_commands(&ctx, &ready).await;

    info!("{}#{} running", ready.user.name, ready.user.id);
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  info!("Tracing initialised");
  let config = config::read_config();
  info!("Config read");
  let intents = GatewayIntents::empty()
    | GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::GUILD_VOICE_STATES;

  info!("Intents: {:?}", intents);

  let mut client = Client::builder(config.token.clone(), intents)
    .event_handler(Handler)
    .application_id(config.application_id)
    .register_songbird()
    .type_map_insert::<constants::HttpKey>(constants::HttpClient::new())
    .type_map_insert::<config::ConfigStorage>(Arc::new(config))
    .await
    .expect("Error creating client");

  if let Err(e) = client.start().await {
    error!("Client error: {:?}", e)
  }
}
