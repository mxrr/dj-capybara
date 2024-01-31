use crate::commands::{playback::VOIPData, text_response, Command};
use serenity::async_trait;
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;
use tracing::error;

pub struct Stop;

#[async_trait]
impl Command for Stop {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let guild_id = voip_data.guild_id;

    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await;
      }
    };

    let handler_lock = match manager.get(guild_id) {
      Some(h) => h,
      None => return text_response(ctx, command, "Not in a voice channel").await,
    };

    let handler = handler_lock.lock().await;
    handler.queue().stop();

    text_response(ctx, command, "Stopped playback and cleared the queue").await
  }

  fn name() -> &'static str {
    "stop"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Stop music and clear the queue")
  }
}
