use std::borrow::Cow;

use anyhow::anyhow;
use poise::serenity_prelude::Member;

use crate::{Context, DiscordCommand, DiscordCommunication, Error};

/// Forfeit a chess match.
#[poise::command(
    slash_command,
    required_permissions = "USE_SLASH_COMMANDS",
    required_bot_permissions = "VIEW_CHANNEL | SEND_MESSAGES | MANAGE_MESSAGES | EMBED_LINKS | READ_MESSAGE_HISTORY | USE_SLASH_COMMANDS | MANAGE_THREADS | CREATE_PUBLIC_THREADS | CREATE_PRIVATE_THREADS | SEND_MESSAGES_IN_THREADS",
    global_cooldown = "5",
    ephemeral = "true"
)]
pub async fn resign(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let resigning_member: Cow<'_, Member>;

    if let Some(member) = ctx.author_member().await {
        resigning_member = member
    } else {
        return Err(anyhow!("Unable to get Member").into());
    }

    ctx.say("Resigning...").await?;

    ctx.data()
        .system_communication_channel
        .0
        .send(DiscordCommunication(
            resigning_member.guild_id,
            DiscordCommand::Resign(Box::new(resigning_member.into_owned())),
        ))?;

    Ok(())
}
