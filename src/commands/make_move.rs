use std::borrow::Cow;

use anyhow::anyhow;
use poise::serenity_prelude::Member;

use crate::{Context, DiscordCommand, DiscordCommunication, Error};

/// Make a chess move.
#[poise::command(
    slash_command,
    required_permissions = "USE_SLASH_COMMANDS",
    required_bot_permissions = "VIEW_CHANNEL | SEND_MESSAGES | MANAGE_MESSAGES | EMBED_LINKS | READ_MESSAGE_HISTORY | USE_SLASH_COMMANDS | MANAGE_THREADS | CREATE_PUBLIC_THREADS | CREATE_PRIVATE_THREADS | SEND_MESSAGES_IN_THREADS",
    ephemeral = "true"
)]
pub async fn make_move(
    ctx: Context<'_>,
    #[description = "The move you wish to make."] chess_move: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let query_member: Cow<'_, Member>;

    if let Some(member) = ctx.author_member().await {
        query_member = member
    } else {
        return Err(anyhow!("Unable to get Member").into());
    }

    ctx.data()
        .system_communication_channel
        .0
        .send(DiscordCommunication(
            query_member.guild_id,
            DiscordCommand::MakeMove(Box::new(query_member.into_owned()), chess_move.clone()),
        ))?;

    ctx.say(format!("Making move: {}...", chess_move)).await?;

    Ok(())
}
