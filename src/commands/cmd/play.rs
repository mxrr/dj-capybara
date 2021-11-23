use crate::commands::{Command, VOIPData, get_source};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::{
  ApplicationCommandInteraction,
  ApplicationCommandInteractionDataOptionValue,
  ApplicationCommandOptionType,
};
use tracing::{error};

pub struct Play;

#[async_trait]
impl Command for Play {

  async fn execute(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let voip_data_f = VOIPData::from(ctx, command);

    let option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => opt_val.clone(),
          None => {
            error!("No options provided");
            return "No search term or URL in request".to_string()
          }
        }
      },
      None => {
        error!("No options provided");
        return "No search term or URL in request".to_string()
      }
    };
  
    let param = if let ApplicationCommandInteractionDataOptionValue::String(s) = option {
      s
    } else {
      error!("Empty URL provided");
      return "No search term or URL in request".to_string()
    };
  
    let voip_data = match voip_data_f.await {
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
  
    let handler = match manager.get(guild_id) {
      Some(h) => h,
      None => {
        let join = manager.join(guild_id, channel_id).await;
        match join.1 {
          Ok(_) => join.0,
          Err(e) => {
            error!("Error joining voice channel: {}", e);
            return "Not in a voice channel".to_string()
          }
        }
      }
    };
  
    let source = match get_source(param).await {
      Ok(s) => s,
      Err(s) => return s,
    };
  
    let title = source
      .metadata
      .title
      .clone()
      .unwrap_or("Missing title".to_string());
  
    let artist = source
      .metadata
      .artist
      .clone()
      .unwrap_or("Missing artist".to_string());
  
    let mut handler_lock = handler.lock().await;
    let _handle = handler_lock.play_only_source(source);
  
    format!("Playing \"{}\" - {}", title, artist)
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("play")
      .description("Play a YouTube video or any music/video file")
      .create_option(|option| {
        option
          .name("search")
          .description("Search term or a link to a YouTube video or a file")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
  }

}