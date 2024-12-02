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

pub const PACKAGE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const RUST_VERSION: &'static str = env!("RUSTC_SEMVER");
pub const LLVM_VERSION: &'static str = env!("RUSTC_LLVM_VERSION");
pub const HOST_TRIPLE: &'static str = env!("RUSTC_HOST_TRIPLE");
pub const BUILD_TIMESTAMP: &'static str = env!("BUILD_TIMESTAMP");
pub const GIT_DESC: &'static str = env!("GIT_COMMIT");
