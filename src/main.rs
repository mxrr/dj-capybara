use serenity::async_trait;
use serenity::client::{
  Client, Context, EventHandler,
  bridge::gateway::GatewayIntents
};
use serenity::model::{
  prelude::*,
  gateway::Activity
};

mod config;
mod constants;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready: Ready) {
    let activity = Activity::listening("test");
    ctx.set_activity(activity).await;
    println!("{}#{} running", ready.user.name, ready.user.discriminator);
  }
}

#[tokio::main]
async fn main() {
  let config = config::read_config();

  let mut client = Client::builder(config.token)
    .event_handler(Handler)
    .intents(
      GatewayIntents::GUILDS | 
      GatewayIntents::GUILD_MESSAGES |
      GatewayIntents::GUILD_VOICE_STATES
    )
    .await
    .expect("Error creating client");
  
  if let Err(e) = client.start().await {
    println!("Client error: {:?}", e);
  }
}
