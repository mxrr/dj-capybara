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

pub struct Resume;

#[async_trait]
impl Command for Resume {
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

    if handler.queue().is_empty() {
      return text_response(ctx, command, "Nothing is paused").await;
    }
    let current = match handler.queue().current() {
      Some(t) => t,
      None => return text_response(ctx, command, "Nothing is paused").await,
    };

    match current.play() {
      Err(e) => {
        error!("Error resuming track: {}", e);
        text_response(ctx, command, "Could not resume").await
      }
      Ok(_) => {
        let metadata = SongMetadata::from_handle(&current).await;
        let title = metadata.title.clone();

        let current_time = current
          .get_info()
          .await
          .map(|info| info.position)
          .unwrap_or_default();
        let current_time = format_duration_live(current_time, &title);
        let duration = format_duration_live(metadata.duration, &title);

        match command
          .edit_response(
            &ctx.http,
            EditInteractionResponse::new().embed(
              CreateEmbed::new()
                .title("Resumed")
                .colour(EMBED_COLOUR)
                .image(metadata.thumbnail)
                .fields(vec![
                  ("Track", title, true),
                  ("Time", format!("{} / {}", current_time, duration), true),
                ]),
            ),
          )
          .await
        {
          Ok(_) => Ok(()),
          Err(e) => Err(e),
        }
      }
    }
  }

  fn name() -> &'static str {
    "resume"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("Resume the currently paused song")
  }
}
