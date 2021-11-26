use crate::commands::{
  Command, 
  text_response,
  utils::remove_md_characters,
};
use serenity::async_trait;
use serenity::client::Context;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType};
use tracing::{error};
use serenity::Error;
use crate::constants::EMBED_COLOUR;

pub struct Info;

#[async_trait]
impl Command for Info {

  async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let option = match command.data.options.get(0) {
      Some(o) => {
        match o.resolved.as_ref() {
          Some(opt_val) => Some(opt_val.clone()),
          None => {
            None
          }
        }
      },
      None => None,
    };

    let user = match option {
      Some(o) => {
        if let ApplicationCommandInteractionDataOptionValue::User(user, _) = o {
          user
        } else {
          error!("Invalid user provided");
          return text_response(ctx, command, "Invalid user provided".to_string()).await
        }
      },
      None => command.user.clone(),
    };

    let nick = if let Some(guild_id) = command.guild_id {
      user
        .nick_in(&ctx.http, guild_id)
        .await
        .unwrap_or(user.name.clone())
    } else {
      user.name.clone()
    };
    

    match command
      .edit_original_interaction_response(&ctx.http, |response| {
        response
          .create_embed(|embed| {
            embed
              .title(user.tag())
              .thumbnail(user.face())
              .colour(EMBED_COLOUR)
              .fields(vec![
                ("Name in guild", remove_md_characters(nick), true),
                ("Joined at", user.created_at().to_string(), true),
                ("Is a bot?", if user.bot { "Yes".to_string() } else { "No".to_string() }, false)
              ])
              .footer(|footer| {
                footer
                  .text(format!("UserID: {}", user.id))
              })
          })
      }).await {
        Ok(_m) => Ok(()),
        Err(e) => Err(e)
      }
  }

  fn info(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
      .name("info")
      .description("View info on your own or someone else's Discord user")
      .create_option(|option| {
        option
          .name("user")
          .description("User you want to view info on, defaults your own user")
          .kind(ApplicationCommandOptionType::User)
          .required(false)
      })
  }

}