use crate::commands::{
  Command, 
  text_response,
  playback::{
    VOIPData, 
    get_source,
  },
};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::{
  ApplicationCommandInteraction,
  ApplicationCommandInteractionDataOptionValue,
  ApplicationCommandOptionType,
};
use tracing::{error};
use serenity::Error;

pub struct Play;

#[async_trait]
impl Command for Play {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => opt_val.clone(),
          None => {
            error!("No options provided");
            return text_response(ctx, command, "No search term or URL in request".to_string()).await
          }
        }
      },
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No search term or URL in request".to_string()).await
      }
    };
  
    let param = if let ApplicationCommandInteractionDataOptionValue::String(s) = option {
      s
    } else {
      error!("Empty URL provided");
      return text_response(ctx, command, "No search term or URL in request".to_string()).await
    };
  
    let voip_data = match VOIPData::from(ctx, &command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await
    };
  
    let guild_id = voip_data.guild_id;
    let channel_id = voip_data.channel_id;
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client".to_string()).await
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
            return text_response(ctx, command, "Not in a voice channel".to_string()).await
          }
        }
      }
    };
  
    let source = match get_source(param).await {
      Ok(s) => s,
      Err(s) => return text_response(ctx, command, s).await,
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
  
    text_response(ctx, command, format!("Playing \"{}\" - {}", title, artist)).await
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