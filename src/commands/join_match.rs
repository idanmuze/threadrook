use std::borrow::Cow;

use anyhow::anyhow;
use poise::serenity_prelude::Member;

use crate::{Context, DiscordCommand, DiscordCommunication, Error};

/// Join the match of any user that is looking for an opponent.
#[poise::command(
    slash_command,
    required_permissions = "USE_SLASH_COMMANDS",
    required_bot_permissions = "VIEW_CHANNEL | SEND_MESSAGES | MANAGE_MESSAGES | EMBED_LINKS | READ_MESSAGE_HISTORY | USE_SLASH_COMMANDS | MANAGE_THREADS | CREATE_PUBLIC_THREADS | CREATE_PRIVATE_THREADS | SEND_MESSAGES_IN_THREADS",
    user_cooldown = "5",
    ephemeral = "true"
)]
pub async fn join_match(
    ctx: Context<'_>,
    #[description = "The member whose match you are joining."] member: Member, // implements ArgumentConvert
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let joining_member: Cow<'_, Member>;

    if let Some(member) = ctx.author_member().await {
        joining_member = member
    } else {
        return Err(anyhow!("Unable to get Member").into());
    }

    ctx.say(format!("Joining {}'s match...", member.user.name))
        .await?;

    ctx.data()
        .system_communication_channel
        .0
        .send(DiscordCommunication(
            member.guild_id,
            DiscordCommand::JoinMatch(
                Box::new(member.to_owned()),
                Box::new(joining_member.into_owned()),
            ),
        ))?;

    Ok(())
}
