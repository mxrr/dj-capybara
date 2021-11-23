use crate::commands::{Command, VOIPData};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use tracing::{error};

pub struct Leave;

#[async_trait]
impl Command for Leave {

  async fn execute(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return s
    };
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return "Error getting voice client".to_string()
      }
    };
  
    let guild_id = voip_data.guild_id;
  
    if manager.get(guild_id).is_some() {
      if let Err(e) = manager.remove(guild_id).await {
        error!("Error leaving voice channel: {}", e);
        return "Error leaving channel".to_string()
      } else {
        return "Left channel".to_string()
      }
    } else {
      "Not in a voice channel".to_string()
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("leave")
      .description("Leave voice channel")
  }

}