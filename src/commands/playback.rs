use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::GuildId;
use songbird::{input::{Restartable, Input}, tracks::TrackHandle};
use tracing::{error};
use std::time::Duration;
use crate::constants::placeholder_img;

pub struct VOIPData {
  pub channel_id: ChannelId,
  pub guild_id: GuildId
}

impl VOIPData {
  pub async fn from(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<VOIPData, String> {
    let guild_from_command = command.guild_id;
    let guild_id = match guild_from_command {
      Some(g_id) => g_id,
      None => {
        error!("Error getting guild from command");
        return Err("Error getting guild information".to_string())
      }
    };
  
    let guild_cache = guild_id.to_guild_cached(&ctx.cache).await;
  
    let channel_id = match guild_cache {
      Some(guild) => {
        let ch = guild
          .voice_states
          .get(&command.member.as_ref().unwrap().user.id.clone())
          .and_then(|vs| vs.channel_id);
        
        match ch {
          Some(c) => c,
          None => {
            error!("Error getting channel id");
            return Err("Join a voice channel first".to_string())
          }
        }
      },
      None => {
        error!("Error getting guild from cache");
        return Err("Error getting guild information".to_string())
      }
    };
  
    let data = VOIPData{channel_id, guild_id};
    Ok(data)
  }
}

#[derive(Clone)]
pub struct SongMetadata {
  pub title: String,
  pub thumbnail: String,
  pub duration: Duration,
  pub url: Option<String>,
}

impl SongMetadata {
  pub fn from_handle(handle: TrackHandle) -> Self {
    let metadata = handle.metadata();

    let thumbnail = metadata
      .thumbnail
      .clone()
      .unwrap_or(placeholder_img());

    let title = metadata
      .title
      .clone()
      .unwrap_or("N/A".to_string());

    let duration = metadata
      .duration
      .clone()
      .unwrap_or_default();

    let url = metadata
      .source_url
      .clone();

    Self {
      title,
      thumbnail,
      duration,
      url,
    }
  }
}



pub async fn get_source(param: String) -> Result<Input, String> {
  if param.contains("https://") {
    match Restartable::ytdl(param, true).await {
      Ok(s) => Ok(s.into()),
      Err(e) => {
        error!("Error fetching music file: {}", e);
        Err("Invalid URL".to_string())
      }
    }
  } else {
    match Restartable::ytdl_search(param.clone(), true).await {
      Ok(s) => Ok(s.into()),
      Err(e) => {
        error!("Error finding youtube video: {}", e);
        Err(format!("Nothing found with \"{}\"", param))
      }
    }
  }
}


pub fn get_queue_length_and_duration(queue: &Vec<TrackHandle>) -> (usize, Duration) {
  (
    queue.len(),
    get_queue_duration(queue),
  )
}

pub fn get_queue_duration(queue: &Vec<TrackHandle>) -> Duration {
  queue
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
    )
}


pub fn format_duration(d: Duration) -> String {
  let s = d.as_secs() % 60;
  let m = (d.as_secs() / 60) % 60;
  let h = (d.as_secs() / 60) / 60;

  format!("{}{}{}",
    if h > 0 { format!("{}h ", h) } else { "".to_string() },
    if m > 0 { format!("{}m ", m) } else { "".to_string() },
    if s > 0 { format!("{}s", s) } else if h > 0 || m > 0 { "".to_string() } else { "n/a".to_string() },
  )
}