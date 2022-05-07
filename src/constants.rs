use serenity::utils::Colour;

pub enum ErrorCodes {
  ConfigFileError = 10,
}

pub fn placeholder_img() -> String { "https://karei.dev/files/christmas.gif".to_string() }

pub const EMBED_COLOUR: Colour = Colour::from_rgb(232, 12, 116);
