use crate::commands::{
  Command, 
  text_response,
  playback::VOIPData
};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use tracing::{error};
use serenity::Error;

pub struct Join;

#[async_trait]
impl Command for Join {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let manager_f = songbird::get(ctx);
    let voip_data = match VOIPData::from(ctx, &command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await
    };
  
    let guild_id = voip_data.guild_id;
    let channel_id = voip_data.channel_id;
  
    let manager = match manager_f.await {
      Some(arc) => arc,
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await
      }
    };
  
    let handler = manager.join(guild_id, channel_id).await;
  
    if let (Some(channel_name), _b) = (channel_id.name(&ctx.cache).await, handler.1.is_ok()) {
      let name = channel_name.clone();
      text_response(ctx, command, format!("Joined channel {}", &name)).await
    } else {
      return text_response(ctx, command, "Couldn't join channel").await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("join")
      .description("Join current voice channel")
  }

}