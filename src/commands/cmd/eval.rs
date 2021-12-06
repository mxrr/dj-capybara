use crate::commands::{Command, text_response};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::{CreateApplicationCommand};
use serenity::model::interactions::application_command::{
  ApplicationCommandInteraction,
  ApplicationCommandOptionType,
  ApplicationCommandInteractionDataOptionValue
};
use tracing::{error};
use serenity::Error;
use evalexpr::eval;
use crate::constants::EMBED_COLOUR;

pub struct Eval;

#[async_trait]
impl Command for Eval {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let expr_option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => opt_val.clone(),
          None => {
            error!("No options provided");
            return text_response(ctx, command, "No expression in request").await
          }
        }
      },
      None => {
        error!("No options provided");
        return text_response(ctx, command, "No expression in request").await
      }
    };

    let expr = if let ApplicationCommandInteractionDataOptionValue::String(expr) = expr_option {
      expr
    } else {
      error!("Invalid option type");
      return text_response(ctx, command, "Malformed expression provided").await
    };

    let desc = match expr.trim() == "help" {
      false => {
        match eval(&expr) {
          Err(e) => {
            error!("Evaluation error: {}", e);
            format!("{}", e)
          },
          Ok(v) => format!("{}", v),
        }
      },
      true => {
        "https://github.com/ISibboI/evalexpr/blob/main/README.md".to_string()
      },
    };

    match command
      .edit_original_interaction_response(&ctx.http, |response| {
        response
          .create_embed(|embed| {
            embed
              .title(expr)
              .colour(EMBED_COLOUR)
              .description(desc)
          })
      }).await {
        Ok(_m) => Ok(()),
        Err(e) => Err(e)
      }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("eval")
      .description("Evaluate an expression")
      .create_option(|option| {
        option
          .name("expression")
          .description("Expression to evaluate (use \"help\" to get a cheatsheet of available functions)")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
  }

}