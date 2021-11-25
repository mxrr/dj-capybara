use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use serenity::{model::{id::GuildId}, prelude::TypeMapKey};
use songbird::tracks::TrackQueue;


pub struct Queue;

impl TypeMapKey for Queue {
  type Value = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
}
