use crate::commands::{Command, playback::{VOIPData, format_duration}, text_response};
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
      let count = handler.queue().len();
      let duration = queue
        .iter()
        .fold(
          Duration::from_secs(0),
          |a, c| {
            let d = c
              .metadata()
              .duration
              .unwrap_or(Duration::from_secs(0));
            a + d
          }
        );

      let current_song_title = queue[0]
        .metadata()
        .title
        .clone()
        .unwrap_or("N/A".to_string());

      let current_song_duration = queue[0]
        .metadata()
        .duration
        .clone()
        .unwrap_or(Duration::from_secs(0));

      let current_song_position = match queue[0]
        .get_info()
        .await {
          Ok(state) => state.position,
          Err(_e) => Duration::from_secs(0),
        };

      let current_song_link = queue[0]
        .metadata()
        .source_url
        .clone();

      let current_song_info = format!(
        "{} \n**[ {} / {} ]**", 
        format_with_url(current_song_title, current_song_link),
        format_duration(current_song_position),
        format_duration(current_song_duration),
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
                    .text(format!("{} songs in queue - {}", count, format_duration(duration)))
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
  let mut length_out = "".to_string();
  for (i, t) in queue.iter().enumerate() {
    if i > 4 { break; }
    if i > 0 {
      let title = t
        .metadata()
        .title
        .clone()
        .unwrap_or("N/A".to_string());

      let url = t
        .metadata()
        .source_url
        .clone();
    
      let duration = format_duration(
        t.metadata()
          .duration
          .unwrap_or(Duration::from_secs(0))
      );

      pos_out.push_str(format!("#{} \n", i).as_str());
      title_out.push_str(format!("{} \n", format_with_url(title, url)).as_str());
      length_out.push_str(format!("{} \n", duration).as_str());
    }
  }
  (pos_out, title_out, length_out)
}