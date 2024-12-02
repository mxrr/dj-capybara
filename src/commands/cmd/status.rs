use crate::{commands::Command, constants};
use constants::EMBED_COLOUR;
use serenity::{
  async_trait,
  builder::{CreateCommand, CreateEmbed, EditInteractionResponse},
  client::Context,
  model::application::CommandInteraction,
  Error,
};

pub struct Status;

#[async_trait]
impl Command for Status {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    command
      .edit_response(
        &ctx.http,
        EditInteractionResponse::new().embed(
          CreateEmbed::new()
            .colour(EMBED_COLOUR)
            .title("Status")
            .fields([
              ("Version", constants::PACKAGE_VERSION, true),
              ("Rust", constants::RUST_VERSION, true),
              ("LLVM", constants::LLVM_VERSION, true),
              ("Commit", constants::GIT_DESC, true),
              ("Host", constants::HOST_TRIPLE, false),
              ("Build", constants::BUILD_TIMESTAMP, false),
            ]),
        ),
      )
      .await?;

    Ok(())
  }

  fn name() -> &'static str {
    "status"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name()).description("display capybara status")
  }
}
