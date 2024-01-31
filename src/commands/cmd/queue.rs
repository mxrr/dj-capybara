use crate::commands::{
  playback::{
    format_duration, format_duration_live, get_queue_length_and_duration, SongMetadata, VOIPData,
  },
  text_response,
  utils::remove_md_characters,
  Command,
};
use crate::constants::EMBED_COLOUR;
use serenity::builder::{CreateCommand, CreateEmbedFooter, EditInteractionResponse};
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::Error;
use serenity::{async_trait, builder::CreateEmbed};
use songbird::tracks::TrackHandle;
use std::time::Duration;
use tracing::error;

pub struct Queue;

#[async_trait]
impl Command for Queue {
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
      Some(h) => h,
      None => return text_response(ctx, command, "Not in a voice channel").await,
    };

    let handler = handler_lock.lock().await;

    if !handler.queue().is_empty() {
      let queue = handler.queue().current_queue();
      let (count, duration) = get_queue_length_and_duration(&queue).await;

      let current_metadata = SongMetadata::from_handle(&queue[0]).await;

      let current_position = match queue[0].get_info().await {
        Ok(state) => state.position,
        Err(e) => {
          error!("Couldn't get track state: {}", e);
          Duration::from_secs(0)
        }
      };

      let current_song_duration =
        format_duration_live(current_metadata.duration, &current_metadata.title);
      let mut live = bool::from(&current_song_duration);

      let current_song_info = format!(
        "{} \n**[ {} / {} ]**",
        format_with_url(
          remove_md_characters(truncate_unicode(&current_metadata.title, 67)),
          current_metadata.url.as_ref()
        ),
        format_duration(current_position),
        current_song_duration,
      );

      let queue_f = format_queue_string(queue).await;

      live = queue_f.3 || live;

      let fields = match handler.queue().len() < 2 {
        true => vec![("Currently playing: ", current_song_info, false)],
        false => vec![
          ("Currently playing: ", current_song_info, false),
          ("Position", queue_f.0, true),
          ("Track", queue_f.1, true),
          ("Duration", queue_f.2, true),
        ],
      };

      let time_left = match live {
        true => "LIVE".to_string(),
        false => format_duration(
          duration
            .checked_sub(current_position)
            .unwrap_or(Duration::from_secs(0)),
        ),
      };

      match command
        .edit_response(
          &ctx.http,
          EditInteractionResponse::new().embed(
            CreateEmbed::new()
              .title("Queue")
              .colour(EMBED_COLOUR)
              .fields(fields)
              .footer(CreateEmbedFooter::new(format!(
                "{} songs in queue - {}",
                count, time_left
              ))),
          ),
        )
        .await
      {
        Ok(_m) => Ok(()),
        Err(e) => Err(e),
      }
    } else {
      text_response(ctx, command, "Queue is empty").await
    }
  }

  fn name() -> &'static str {
    "queue"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("View currently queued songs")
  }
}

fn format_with_url(title: String, url: Option<&String>) -> String {
  if let Some(link) = url {
    format!("[{}]({})", title, link)
  } else {
    title
  }
}

fn truncate_unicode(text: &str, max_chars: usize) -> String {
  match text.char_indices().nth(max_chars) {
    None => text.to_string(),
    Some((i, _)) => {
      let mut valid_index = i;
      while !text.is_char_boundary(valid_index) {
        valid_index -= 1;
      }
      let trimmed = &text[..valid_index];
      format!("{}...", trimmed)
    }
  }
}

async fn format_queue_string(queue: Vec<TrackHandle>) -> (String, String, String, bool) {
  let mut pos_out = "".to_string();
  let mut title_out = "".to_string();
  let mut duration_out = "".to_string();
  let mut live = false;
  for (i, handle) in queue.iter().enumerate().skip(1).take(4) {
    let metadata = SongMetadata::from_handle(handle).await;
    let title_trimmed = truncate_unicode(&metadata.title, 37);
    let title = format_with_url(remove_md_characters(title_trimmed), metadata.url.as_ref());

    let duration = format_duration_live(metadata.duration, &metadata.title);
    live = bool::from(&duration) || live;

    pos_out.push_str(format!("#{} \n", i).as_str());
    title_out.push_str(format!("{} \n", title).as_str());
    duration_out.push_str(format!("{} \n", duration).as_str());
  }
  (pos_out, title_out, duration_out, live)
}
