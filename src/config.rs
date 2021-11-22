use crate::constants;

pub struct Config {
  pub token: String,
}

pub fn read_config() -> Config {
  match dotenv::dotenv() {
    Ok(c) => println!("Loaded .env {:?}", c),
    Err(e) => {
      println!("Error {:?}", e);
      std::process::exit(constants::ErrorCodes::ConfigFileError as i32);
    }
  }

  let token = std::env::var("TOKEN").expect("Missing bot token in .env");

  return Config {
    token
  }
}