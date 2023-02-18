use crate::commands::{
  playback::{format_duration_live, VOIPData},
  text_response, Command,
};
use crate::constants::EMBED_COLOUR;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::Error;
use std::time::Duration;
use tracing::error;

pub struct Skip;

#[async_trait]
impl Command for Skip {
  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let voip_data = match VOIPData::from(ctx, &command).await {
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

    if handler.queue().len() > 0 {
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
          let title = current
            .metadata()
            .title
            .clone()
            .unwrap_or("N/A".to_string());

          let length = format_duration_live(
            current
              .metadata()
              .duration
              .unwrap_or(Duration::from_secs(0)),
            title.clone(),
          )
          .0;

          match command
            .edit_original_interaction_response(&ctx.http, |response| {
              response.embed(|embed| {
                embed
                  .title("Skipped")
                  .colour(EMBED_COLOUR)
                  .fields(vec![("Track", title, true), ("Length", length, true)])
              })
            })
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

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("skip")
      .description("Skip the currently playing song")
  }
}
