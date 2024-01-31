use crate::commands::playback::{SongMetadata, VOIPData};
use crate::commands::{text_response, Command};
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::client::Context;
use serenity::model::application::{CommandInteraction, CommandOptionType, ResolvedValue};
use serenity::Error;
use tracing::error;

pub struct Seek;

const SECONDS_OPTION_NAME: &str = "seconds";

#[async_trait]
impl Command for Seek {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
    };

    let timestamp = match command
      .data
      .options()
      .iter()
      .find(|e| e.name == SECONDS_OPTION_NAME)
    {
      Some(o) => match o.value {
        ResolvedValue::Number(time) => std::time::Duration::from_secs_f64(time),
        _ => {
          error!("Invalid option type");
          return text_response(ctx, command, "Malformed timestamp provided").await;
        }
      },
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No timestamp in request").await;
      }
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
      Some(h) => {
        if voip_data.compare_to_call(&h).await {
          h
        } else {
          return text_response(ctx, command, "You're not in the voice channel").await;
        }
      }
      None => return text_response(ctx, command, "Not in a voice channel").await,
    };

    let handler = handler_lock.lock().await;

    if handler.queue().is_empty() {
      return text_response(ctx, command, "Nothing playing").await;
    }

    let current = match handler.queue().current() {
      Some(t) => t,
      None => return text_response(ctx, command, "Nothing playing").await,
    };

    let metadata = SongMetadata::from_handle(&current).await;
    let current_duration = metadata.duration;

    if current_duration > std::time::Duration::default() && timestamp >= current_duration {
      return text_response(
        ctx,
        command,
        format!(
          "Cannot seek past song's end (max: {}s)",
          current_duration.as_secs()
        ),
      )
      .await;
    }

    let current_position = match current.get_info().await {
      Ok(state) => state.position,
      Err(e) => {
        error!("Couldn't get TrackState: {}", e);
        std::time::Duration::default()
      }
    };

    if timestamp <= current_position {
      return text_response(
        ctx,
        command,
        format!(
          "Cannot seek backwards (current: {}s)",
          current_position.as_secs()
        ),
      )
      .await;
    }

    match current.seek(timestamp).result_async().await {
      Ok(_) => text_response(ctx, command, format!("Seeked to {:?}", timestamp)).await,
      Err(e) => {
        error!("Error while seeking: {}", e);
        text_response(ctx, command, "Unknown error seeking").await
      }
    }
  }

  fn name() -> &'static str {
    "seek"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name())
      .description("Seek the currently playing song")
      .add_option(
        CreateCommandOption::new(
          CommandOptionType::Number,
          SECONDS_OPTION_NAME,
          "Jump to this point in the audio",
        )
        .required(true),
      )
  }
}
