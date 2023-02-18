use crate::commands::{text_response, utils::remove_md_characters, Command};
use crate::constants::EMBED_COLOUR;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOptionValue,
};
use serenity::model::prelude::command::CommandOptionType;
use serenity::Error;
use tracing::error;

pub struct Info;

#[async_trait]
impl Command for Info {
    async fn execute(ctx: &Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
        let option = match command.data.options.get(0) {
            Some(o) => match o.resolved.as_ref() {
                Some(opt_val) => Some(opt_val.clone()),
                None => None,
            },
            None => None,
        };

        let user = match option {
            Some(o) => {
                if let CommandDataOptionValue::User(user, _) = o {
                    match ctx.http.get_user(user.id.0).await {
                        Err(e) => {
                            error!("Couldn't fetch user {}", e);
                            user
                        }
                        Ok(u) => u,
                    }
                } else {
                    error!("Invalid user provided");
                    return text_response(ctx, command, "Invalid user provided").await;
                }
            }
            None => command.user.clone(),
        };

        let (nick, avatar) = if let Some(guild_id) = command.guild_id {
            match ctx.http.get_member(guild_id.0, user.id.0).await {
                Err(e) => {
                    error!("Couldn't fetch member {}", e);
                    (user.name.clone(), user.face())
                }
                Ok(member) => (
                    member.display_name().into_owned(),
                    member.avatar_url().unwrap_or(user.face()),
                ),
            }
        } else {
            (user.name.clone(), user.face())
        };

        let join_time =
            chrono::NaiveDateTime::from_timestamp(user.created_at().unix_timestamp(), 0);
        let join_time_string = join_time.format("%d %B %Y, %H:%M:%S").to_string();

        let user_colour = user.accent_colour.unwrap_or(EMBED_COLOUR);

        let banner_url = user.banner_url().unwrap_or_default();

        match command
            .edit_original_interaction_response(&ctx.http, |response| {
                response.embed(|embed| {
                    embed
                        .title(remove_md_characters(nick))
                        .image(banner_url)
                        .thumbnail(avatar)
                        .colour(user_colour)
                        .fields(vec![
                            ("User", user.tag(), true),
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
                        .footer(|footer| footer.text(format!("UserID: {}", user.id)))
                })
            })
            .await
        {
            Ok(_m) => Ok(()),
            Err(e) => Err(e),
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
                    .kind(CommandOptionType::User)
                    .required(false)
            })
    }
}
