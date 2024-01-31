pub use reqwest::Client as HttpClient;
use serenity::model::Colour;
use serenity::prelude::TypeMapKey;

pub enum ErrorCodes {
  ConfigFileError = 10,
}

pub fn placeholder_img() -> String {
  "https://karei.dev/files/capybara-default.jpg".to_string()
}

pub const EMBED_COLOUR: Colour = Colour::from_rgb(232, 12, 116);

pub struct HttpKey;

impl TypeMapKey for HttpKey {
  type Value = HttpClient;
}
