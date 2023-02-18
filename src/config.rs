use crate::constants;
use serenity::{model::id::GuildId, prelude::TypeMapKey};
use std::sync::Arc;
use tracing::{error, info};

pub struct ConfigStorage;

impl TypeMapKey for ConfigStorage {
  type Value = Arc<Config>;
}

pub struct Config {
  pub token: String,
  pub application_id: u64,
  pub guild_id: Option<GuildId>,
}

pub fn read_config() -> Config {
  match dotenv::dotenv() {
    Ok(c) => info!("Loaded .env {:?}", c),
    Err(e) => {
      error!("Error {:?}", e);
      std::process::exit(constants::ErrorCodes::ConfigFileError as i32);
    }
  }

  let token = std::env::var("TOKEN").expect("Missing bot token in .env");

  let application_id = std::env::var("APP_ID")
    .expect("APP_ID missing from .env")
    .parse()
    .expect("Invalid APP_ID");

  let guild_id = match std::env::var("GUILD_ID") {
    Ok(id) => match id.parse::<u64>() {
      Ok(g) => {
        info!("Registering commands on GUILD_ID({})", g);
        Some(GuildId(g))
      }
      Err(e) => {
        error!("Error parsing GUILD_ID({}), registering globally", id);
        error!("ParseError: {:?}", e);
        None
      }
    },
    Err(_e) => {
      info!("No GUILD_ID in .env, registering globally");
      None
    }
  };

  Config {
    token,
    application_id,
    guild_id,
  }
}
