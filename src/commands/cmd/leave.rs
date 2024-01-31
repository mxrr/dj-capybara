use crate::commands::text_response;
use crate::commands::{playback::VOIPData, Command};
use serenity::async_trait;
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;
use tracing::error;

pub struct Leave;

#[async_trait]
impl Command for Leave {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await;
      }
    };

    let guild_id = voip_data.guild_id;

    if let Some(handler_lock) = manager.get(guild_id) {
      if !voip_data.compare_to_call(&handler_lock).await {
        return text_response(ctx, command, "You're not in the voice channel").await;
      }

      if let Err(e) = manager.remove(guild_id).await {
        error!("Error leaving voice channel: {}", e);
        return text_response(ctx, command, "Error leaving channel").await;
      } else {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
        return text_response(ctx, command, "Left channel").await;
      }
    } else {
      text_response(ctx, command, "Not in a voice channel").await
    }
  }

  fn name() -> &'static str {
    "leave"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Leave voice channel")
  }
}
