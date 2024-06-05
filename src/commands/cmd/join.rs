use crate::commands::{
  playback::{ClientDisconnectEvent, VOIPData},
  text_response, Command,
};
use serenity::async_trait;
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;
use songbird::{CoreEvent, Event};
use tracing::error;

pub struct Join;

#[async_trait]
impl Command for Join {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let manager_f = songbird::get(ctx);
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let guild_id = voip_data.guild_id;
    let channel_id = voip_data.channel_id;

    let manager = match manager_f.await {
      Some(arc) => arc,
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await;
      }
    };

    let handler = match manager.join(guild_id, channel_id).await {
      Ok(c) => c,
      Err(e) => {
        error!("Error joining channel: {}", e);
        return text_response(ctx, command, "Couldn't join channel").await;
      }
    };

    let mut handler_lock = handler.lock().await;
    handler_lock.add_global_event(
      Event::Core(CoreEvent::ClientDisconnect),
      ClientDisconnectEvent { ctx: ctx.clone() },
    );

    match channel_id.name(&ctx.http).await {
      Ok(channel_name) => {
        text_response(ctx, command, format!("Joined channel {}", channel_name)).await
      }
      Err(e) => {
        error!("Error getting channel_name: {}", e);
        return text_response(ctx, command, "Couldn't join channel").await;
      }
    }
  }

  fn name() -> &'static str {
    "join"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Join current voice channel")
  }
}
