use crate::commands::{
  Command, 
  playback::{
    VOIPData, 
    format_duration,
    SongMetadata,
    get_queue_length_and_duration,
  }, 
  text_response,
  utils::remove_md_characters,
};
use serenity::{async_trait};
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use songbird::tracks::TrackHandle;
use tracing::error;
use serenity::Error;
use std::time::Duration;
use crate::constants::EMBED_COLOUR;

pub struct Queue;

#[async_trait]
impl Command for Queue {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
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
  
    let handler_lock = match manager.get(guild_id) {
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

    let handler = handler_lock.lock().await;

    if handler.queue().len() > 0 {
      let queue = handler.queue().current_queue();
      let (count, duration) = get_queue_length_and_duration(&queue);


      let current_metadata = SongMetadata::from_handle(queue[0].clone());

      let current_position = match queue[0]
        .get_info()
        .await {
          Ok(state) => state.position,
          Err(e) => {
            error!("Couldn't get track state: {}", e);
            Duration::from_secs(0)
          },
        };

      let current_song_info = format!(
        "{} \n**[ {} / {} ]**", 
        format_with_url(remove_md_characters(current_metadata.title), current_metadata.url),
        format_duration(current_position),
        format_duration(current_metadata.duration),
      );
      
      let queue_f = format_queue_string(queue);

      let fields = match handler.queue().len() < 2 {
        true => vec![("Currently playing: ", current_song_info, false),],
        false => vec![
          ("Currently playing: ", current_song_info, false),
          ("Position", queue_f.0, true),
          ("Track", queue_f.1, true),
          ("Duration", queue_f.2, true),
        ]
      };

      match command
        .edit_original_interaction_response(&ctx.http, |response| {
          response
            .create_embed(|embed| {
              embed
                .title("Queue")
                .colour(EMBED_COLOUR)
                .fields(fields)
                .footer(|footer| {
                  footer
                    .text(
                      format!("{} songs in queue - {}", 
                        count, 
                        format_duration(duration - current_position)
                      ))
                })
            })
        }).await {
          Ok(_m) => Ok(()),
          Err(e) => Err(e)
        }
    } else {
      text_response(ctx, command, "Queue is empty".to_string()).await
    }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("queue")
      .description("View currently queued songs")
  }

}


fn format_with_url(title: String, url: Option<String>) -> String {
  if let Some(link) = url {
    format!("[{}]({})", title, link)
  } else {
    title
  }
}

fn format_queue_string(queue: Vec<TrackHandle>) -> (String, String, String) {
  let mut pos_out = "".to_string();
  let mut title_out = "".to_string();
  let mut duration_out = "".to_string();
  for (i, t) in queue.iter().enumerate() {
    if i > 4 { break; }
    if i > 0 {
      let metadata = SongMetadata::from_handle(t.clone());
      let mut title = format_with_url(
          remove_md_characters(
            if metadata.title.len() > 40 {
              let mut t = metadata
                .title
                .split_at(37)
                .0
                .to_string();
              t.push_str("...");
              t
            } else {
              metadata.title
            }
          ), 
          metadata.url
      );
      title.shrink_to(16);

      pos_out.push_str(format!("#{} \n", i).as_str());
      title_out.push_str(format!("{} \n", title).as_str());
      duration_out.push_str(format!("{} \n", format_duration(metadata.duration)).as_str());
    }
  }
  (pos_out, title_out, duration_out)
}