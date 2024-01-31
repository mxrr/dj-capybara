use crate::commands::{
  playback::{format_duration_live, SongMetadata, VOIPData},
  text_response, Command,
};
use crate::constants::EMBED_COLOUR;
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;
use serenity::{
  async_trait,
  builder::{CreateEmbed, EditInteractionResponse},
};
use tracing::error;

pub struct Skip;

#[async_trait]
impl Command for Skip {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, command).await {
      Ok(v) => v,
      Err(s) => return text_response(ctx, command, s).await,
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

    if !handler.queue().is_empty() {
      let current = match handler.queue().current() {
        Some(t) => t,
        None => return text_response(ctx, command, "Nothing to skip").await,
      };

      match handler.queue().skip() {
        Err(e) => {
          error!("Error skipping track: {}", e);
          text_response(ctx, command, "Nothing to skip").await
        }
        Ok(_) => {
          let metadata = SongMetadata::from_handle(&current).await;
          let title = metadata.title.clone();

          let length = format_duration_live(metadata.duration, &title);

          match command
            .edit_response(
              &ctx.http,
              EditInteractionResponse::new().embed(
                CreateEmbed::new()
                  .title("Skipped")
                  .colour(EMBED_COLOUR)
                  .fields(vec![
                    ("Track", title, true),
                    ("Length", length.to_string(), true),
                  ]),
              ),
            )
            .await
          {
            Ok(_m) => Ok(()),
            Err(e) => Err(e),
          }
        }
      }
    } else {
      text_response(ctx, command, "Nothing to skip").await
    }
  }

  fn name() -> &'static str {
    "skip"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Skip the currently playing song")
  }
}
