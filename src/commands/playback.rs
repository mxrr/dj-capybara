use crate::constants::placeholder_img;
use regex::Regex;
use serenity::client::Context;
use serenity::model::application::CommandInteraction;
use serenity::model::id::ChannelId;
use serenity::model::prelude::GuildId;
use serenity::prelude::Mutex;
use songbird::{
  input::{Compose, YoutubeDl},
  tracks::TrackHandle,
  typemap::TypeMapKey,
};
use std::{sync::Arc, time::Duration};
use tracing::error;

pub struct VOIPData {
  pub channel_id: ChannelId,
  pub guild_id: GuildId,
}

impl VOIPData {
  pub async fn from(ctx: &Context, command: &CommandInteraction) -> Result<VOIPData, String> {
    let guild_from_command = command.guild_id;
    let guild_id = match guild_from_command {
      Some(g_id) => g_id,
      None => {
        error!("Error getting guild from command");
        return Err("Error getting guild information".to_string());
      }
    };

    let guild_cache = guild_id.to_guild_cached(&ctx.cache);

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
            return Err("Join a voice channel first".to_string());
          }
        }
      }
      None => {
        error!("Error getting guild from cache");
        return Err("Error getting guild information".to_string());
      }
    };

    let data = VOIPData {
      channel_id,
      guild_id,
    };
    Ok(data)
  }

  pub async fn compare_to_call(&self, call: &Arc<Mutex<songbird::Call>>) -> bool {
    call
      .lock()
      .await
      .current_channel()
      .map_or(0u64, |id| id.0.into())
      == self.channel_id.get()
  }
}

#[derive(Clone)]
pub struct SongMetadata {
  pub title: String,
  pub thumbnail: String,
  pub duration: Duration,
  pub url: Option<String>,
}

pub struct SongMetadataKey;

impl TypeMapKey for SongMetadataKey {
  type Value = SongMetadata;
}

impl SongMetadata {
  pub async fn from_source(source: &mut YoutubeDl) -> Self {
    let metadata = match source.aux_metadata().await {
      Ok(m) => m,
      Err(e) => {
        error!("Error getting metadata: {}", e);
        return Self {
          title: "N/A".to_string(),
          thumbnail: placeholder_img(),
          duration: Duration::default(),
          url: None,
        };
      }
    };

    let thumbnail = metadata.thumbnail.clone().unwrap_or_else(placeholder_img);

    let title = metadata.title.clone().unwrap_or_else(|| "N/A".to_string());

    let duration = metadata.duration.unwrap_or_default();

    let url = metadata.source_url.clone();

    Self {
      title,
      thumbnail,
      duration,
      url,
    }
  }

  pub async fn from_handle(handle: &TrackHandle) -> SongMetadata {
    let data = handle.typemap().read().await;
    data
      .get::<SongMetadataKey>()
      .expect("Metadata not found")
      .clone()
  }
}

pub fn get_source(client: crate::constants::HttpClient, param: String) -> YoutubeDl {
  if param.contains("https://") {
    YoutubeDl::new(client, param)
  } else {
    YoutubeDl::new_search(client, param)
  }
}

pub async fn get_queue_length_and_duration(queue: &Vec<TrackHandle>) -> (usize, Duration) {
  (queue.len(), get_queue_duration(queue).await)
}

pub async fn get_queue_duration(queue: &[TrackHandle]) -> Duration {
  let mut total_duration = Duration::from_secs(0);
  for handle in queue {
    let metadata = SongMetadata::from_handle(handle).await;
    total_duration += metadata.duration;
  }
  total_duration
}

pub enum DurationFormat {
  Live(),
  Normal(String),
}

impl std::fmt::Display for DurationFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Normal(s) => write!(f, "{}", s),
      Self::Live() => write!(f, "LIVE"),
    }
  }
}

impl From<DurationFormat> for bool {
  fn from(duration_format: DurationFormat) -> Self {
    match duration_format {
      DurationFormat::Live() => true,
      DurationFormat::Normal(_) => false,
    }
  }
}
impl From<&DurationFormat> for bool {
  fn from(duration_format: &DurationFormat) -> Self {
    match duration_format {
      DurationFormat::Live() => true,
      DurationFormat::Normal(_) => false,
    }
  }
}

pub fn format_duration_live(d: Duration, t: &str) -> DurationFormat {
  let re = Regex::new(r".+([0-9]){4}-([0-9]){2}-([0-9]){2}\W([0-9]){1,2}:([0-9]){1,2}$")
    .expect("Failed to compile regex");
  if re.is_match(t) {
    DurationFormat::Live()
  } else {
    DurationFormat::Normal(format_duration(d))
  }
}

pub fn format_duration(d: Duration) -> String {
  let s = d.as_secs() % 60;
  let m = (d.as_secs() / 60) % 60;
  let h = (d.as_secs() / 60) / 60;

  format!(
    "{}{}{}",
    if h > 0 {
      format!("{}h ", h)
    } else {
      "".to_string()
    },
    if m > 0 {
      format!("{}m ", m)
    } else {
      "".to_string()
    },
    if s > 0 {
      format!("{}s", s)
    } else if h > 0 || m > 0 {
      "".to_string()
    } else {
      "n/a".to_string()
    },
  )
}
