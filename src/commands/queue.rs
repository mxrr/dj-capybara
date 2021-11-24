use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use serenity::{model::{id::GuildId}, prelude::TypeMapKey};
use songbird::{input::Input, tracks::{TrackHandle, TrackQueue}};


pub struct Queue;

impl TypeMapKey for Queue {
  type Value = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
}

#[derive(Debug)]
pub struct GuildQueue {
  pub guild_id: GuildId,
  pub current_song: TrackHandle,
  pub queue: Vec<Input>,
  pub queue_duration: Duration,
}


impl GuildQueue {
  pub fn new(guild_id: GuildId, current_song: TrackHandle, queue: Vec<Input>) -> Self {
    let duration = get_queue_duration(&current_song, &queue);
    Self {
      guild_id,
      current_song,
      queue,
      queue_duration: duration,
    }
  }


  fn update_queue_duration(&mut self) {
    self.queue_duration = get_queue_duration(&self.current_song, &self.queue)
  }
}


fn get_queue_duration(current_song: &TrackHandle, queue: &Vec<Input>) -> Duration {
  let duration = queue
  .iter()
  .fold(
    current_song
          .metadata()
          .duration
          .unwrap_or(Duration::from_secs(0)), 
    |c, n| {
      c + n.metadata.duration.unwrap_or(Duration::from_secs(0))
    });

  duration
}