use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use serenity::{model::{id::GuildId}, prelude::TypeMapKey};
use songbird::{input::Input, tracks::TrackHandle};


pub struct Queue;

impl TypeMapKey for Queue {
  type Value = Arc<Mutex<HashMap<GuildId, GuildQueue>>>;
}

pub struct GuildQueue {
  pub guild_id: GuildId,
  pub current_song: TrackHandle,
  pub queue: Vec<Input>,
  pub queue_duration: Duration,
}


impl GuildQueue {
  fn update_queue_duration(&mut self) {
    let duration = self
      .queue
      .iter()
      .fold(
        self
          .current_song
          .metadata()
          .duration
          .unwrap_or(Duration::from_secs(0)), 
        |c, n| {
          c + n.metadata.duration.unwrap_or(Duration::from_secs(0))
        });

    self.queue_duration = duration;
  }
}