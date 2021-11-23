use crate::commands::{Command, VOIPData};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use tracing::{error};

pub struct Join;

#[async_trait]
impl Command for Join {

  async fn execute(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return s
    };
  
    let guild_id = voip_data.guild_id;
    let channel_id = voip_data.channel_id;
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return "Error getting voice client".to_string()
      }
    };
  
    let _handler = manager.join(guild_id, channel_id).await;
  
    if let Some(channel_name) = channel_id.name(&ctx.cache).await {
      format!("Joined channel {}", channel_name)
    } else {
      "Joined channel".to_string()
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("join")
      .description("Join current voice channel")
  }

}