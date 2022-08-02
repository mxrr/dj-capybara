use crate::commands::playback::VOIPData;
use crate::commands::{Command, text_response};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType, ApplicationCommandInteractionDataOptionValue};
use tracing::{error};
use serenity::Error;

pub struct Seek;

#[async_trait]
impl Command for Seek {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, &command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await
    };

    let seek_option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => opt_val.clone(),
          None => {
            error!("No options provided");
            return text_response(ctx, command, "No timestamp in request").await
          }
        }
      },
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No timestamp in request").await
      }
    };

    let timestamp = if let ApplicationCommandInteractionDataOptionValue::Number(time) = seek_option {
      std::time::Duration::from_secs_f64(time)
    } else {
      error!("Invalid option type");
      return text_response(ctx, command, "Malformed expression provided").await
    };

    let guild_id = voip_data.guild_id;
  
    let manager = match songbird::get(ctx).await {
      Some(arc) => arc.clone(),
      None => {
        error!("Error with songbird client");
        return text_response(ctx, command, "Error getting voice client").await
      }
    };
  
    let handler_lock = match manager.get(guild_id) {
      Some(h) => {
        if voip_data.compare_to_call(&h).await { h }
        else { return text_response(ctx, command, "You're not in the voice channel").await }
      },
      None => {
          return text_response(ctx, command, "Not in a voice channel").await
        }
    };

    let handler = handler_lock.lock().await;

    if handler.queue().len() > 0 {
      let current = match handler.queue().current() {
        Some(t) => t,
        None => return text_response(ctx, command, "Nothing playing").await
      };

      let current_duration = current
        .metadata()
        .duration
        .unwrap_or_default();

      if current_duration > std::time::Duration::default() && timestamp >= current_duration {
        return text_response(ctx, command, format!("Cannot seek past song's end ({:?})", current_duration)).await
      }

      if !current.is_seekable() { 
        return text_response(ctx, command, "Unknown error while seeking").await 
      }

      match current.seek_time(timestamp) {
        Ok(_) => text_response(ctx, command, format!("Seeked to {:?}", timestamp)).await,
        Err(e) => text_response(ctx, command, e).await
      }
    } else {
      text_response(ctx, command, "Nothing playing").await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
    .name("seek")
    .description("Seek the currently playing song")
    .create_option(|option| {
      option
        .name("seconds")
        .description("Jump to this point in the audio")
        .kind(ApplicationCommandOptionType::Number)
        .required(true)
    })
  }

}