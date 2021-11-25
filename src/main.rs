use serenity::async_trait;
use serenity::client::{
  Client, Context, EventHandler,
  bridge::gateway::GatewayIntents
};
use serenity::model::{
  prelude::*,
  gateway::Activity
};
use songbird::SerenityInit;
use tracing::{info, error};
use std::sync::Arc;

mod config;
mod constants;
mod commands;


struct Handler;

#[async_trait]
impl EventHandler for Handler {

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
      commands::handle_commands(&ctx, command.clone()).await;
    }
  }


  async fn ready(&self, ctx: Context, ready: Ready) {
    let activity = Activity::playing("with 🍊");
    ctx.set_activity(activity).await;

    commands::register_commands(&ctx, &ready).await;

    info!("{}#{} running", ready.user.name, ready.user.discriminator);
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  let config = config::read_config();

  let mut client = Client::builder(config.token.clone())
    .event_handler(Handler)
    .application_id(config.application_id.clone())
    .intents(
      GatewayIntents::GUILDS | 
      GatewayIntents::GUILD_MESSAGES |
      GatewayIntents::GUILD_VOICE_STATES
    )
    .register_songbird()
    .await
    .expect("Error creating client");

  {
    use config::ConfigStorage;
    let mut data = client.data.write().await;
    data.insert::<ConfigStorage>(Arc::new(config));
  }
  
  if let Err(e) = client.start().await {
    error!("Client error: {:?}", e);
  }
}
