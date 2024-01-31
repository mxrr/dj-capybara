use crate::commands::{text_response, utils::remove_md_characters, Command};
use crate::constants::EMBED_COLOUR;
use serenity::{
  all::ResolvedValue,
  async_trait,
  builder::{
    CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, EditInteractionResponse,
  },
  client::Context,
  model::application::{CommandInteraction, CommandOptionType},
  Error,
};
use tracing::error;

pub struct Info;

const USER_OPTION_NAME: &str = "user";

#[async_trait]
impl Command for Info {
  async fn execute(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
    let option = command
      .data
      .options()
      .iter()
      .find(|e| e.name == USER_OPTION_NAME)
      .map(|o| o.to_owned());

    let user_id = match option {
      Some(o) => {
        if let ResolvedValue::User(user, _) = o.value {
          user.id
        } else {
          error!("Invalid user provided");
          return text_response(ctx, command, "Invalid user provided").await;
        }
      }
      None => command.user.id,
    };

    let user = match ctx.http.get_user(user_id).await {
      Err(e) => {
        error!("Couldn't fetch user {}", e);
        return text_response(ctx, command, "Couldn't fetch user").await;
      }
      Ok(u) => u,
    };

    let (nick, avatar) = if let Some(guild_id) = command.guild_id {
      match ctx.http.get_member(guild_id, user.id).await {
        Err(e) => {
          error!("Couldn't fetch member {}", e);
          (user.name.clone(), user.face())
        }
        Ok(member) => (
          member.display_name().to_string(),
          member.avatar_url().unwrap_or_else(|| user.face()),
        ),
      }
    } else {
      (user.name.clone(), user.face())
    };

    let join_time =
      chrono::NaiveDateTime::from_timestamp_opt(user.created_at().unix_timestamp(), 0)
        .unwrap_or_default();
    let join_time_string = join_time.format("%d %B %Y, %H:%M:%S").to_string();

    let user_colour = user.accent_colour.unwrap_or(EMBED_COLOUR);

    let banner_url = user.banner_url().unwrap_or_default();

    match command
      .edit_response(
        &ctx.http,
        EditInteractionResponse::new().embed(
          CreateEmbed::new()
            .title(remove_md_characters(nick))
            .image(banner_url)
            .thumbnail(avatar)
            .colour(user_colour)
            .fields(vec![
              ("User", user.name, true),
              ("Joined at", join_time_string, true),
              (
                "Is a bot?",
                if user.bot {
                  "Yes".to_string()
                } else {
                  "No".to_string()
                },
                false,
              ),
            ])
            .footer(CreateEmbedFooter::new(format!("UserID: {}", user.id))),
        ),
      )
      .await
    {
      Ok(_m) => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn name() -> &'static str {
    "info"
  }

  fn info() -> CreateCommand {
    CreateCommand::new(Self::name())
      .description("View info on your own or someone else's Discord user")
      .add_option(
        CreateCommandOption::new(
          CommandOptionType::User,
          USER_OPTION_NAME,
          "User you want to view info on, defaults your own user",
        )
        .required(false),
      )
  }
}
