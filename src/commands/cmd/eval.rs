use crate::commands::utils::remove_md_characters;
use crate::commands::{text_response, Command};
use crate::constants::EMBED_COLOUR;
use evalexpr::eval;
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed, EditInteractionResponse};
use serenity::client::Context;
use serenity::model::application::{CommandInteraction, CommandOptionType, ResolvedValue};
use serenity::Error;
use tracing::error;

pub struct Eval;

const EXPRESSION_OPTION_NAME: &str = "expression";

#[async_trait]
impl Command for Eval {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let expr = match command
      .data
      .options()
      .iter()
      .find(|e| e.name == EXPRESSION_OPTION_NAME)
    {
      Some(o) => {
        if let ResolvedValue::String(expr) = o.value {
          expr.to_string()
        } else {
          error!("Invalid option type");
          return text_response(ctx, command, "Malformed expression provided").await;
        }
      }
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No expression in request").await;
      }
    };

    let desc = match expr.trim() == "help" {
      false => match eval(&expr) {
        Err(e) => {
          error!("Evaluation error: {}", e);
          format!("{}", e)
        }
        Ok(v) => format!("{}", v),
      },
      true => "https://github.com/ISibboI/evalexpr/blob/main/README.md".to_string(),
    };

    match command
      .edit_response(
        &ctx.http,
        EditInteractionResponse::new().embed(
          CreateEmbed::new()
            .title(remove_md_characters(expr))
            .colour(EMBED_COLOUR)
            .description(desc),
        ),
      )
      .await
    {
      Ok(_m) => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn name() -> &'static str {
    "eval"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name())
      .description("Evaluate an expression")
      .add_option(
        CreateCommandOption::new(
          CommandOptionType::String,
          EXPRESSION_OPTION_NAME,
          "Expression to evaluate (use \"help\" to get a cheatsheet of available functions)",
        )
        .required(true),
      )
  }
}
