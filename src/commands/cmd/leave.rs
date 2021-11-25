use crate::commands::text_response;
use crate::commands::{
  Command, 
  playback::VOIPData
};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use tracing::{error};
use serenity::Error;

pub struct Leave;

#[async_trait]
impl Command for Leave {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, &command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await
    };
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client".to_string()).await
      }
    };
  
    let guild_id = voip_data.guild_id;
  
    if let Some(handler_lock) = manager.get(guild_id) {
      if let Err(e) = manager.remove(guild_id).await {
        error!("Error leaving voice channel: {}", e);
        return text_response(ctx, command, "Error leaving channel".to_string()).await
      } else {
        let handler = handler_lock.lock().await;
        handler.queue().stop();
        return text_response(ctx, command, "Left channel".to_string()).await
      }
    } else {
      text_response(ctx, command, "Not in a voice channel".to_string()).await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("leave")
      .description("Leave voice channel")
  }

}