use crate::constants;
use tracing::{info, error};

pub struct Config {
  pub token: String,
  pub guild_id: String,
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
  let guild_id = std::env::var("GUILD_ID").unwrap_or("".to_string());

  return Config {
    token,
    guild_id
  }
}