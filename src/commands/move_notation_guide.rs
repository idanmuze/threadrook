use crate::{Context, Error};

/// A quick guide on chess move notation.
#[poise::command(
    slash_command,
    required_permissions = "USE_SLASH_COMMANDS",
    required_bot_permissions = "VIEW_CHANNEL | SEND_MESSAGES | MANAGE_MESSAGES | EMBED_LINKS | READ_MESSAGE_HISTORY | USE_SLASH_COMMANDS | MANAGE_THREADS | CREATE_PUBLIC_THREADS | CREATE_PRIVATE_THREADS | SEND_MESSAGES_IN_THREADS",
    user_cooldown = "5",
    ephemeral = "true"
)]
pub async fn move_notation_guide(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    ctx.send(|m| {
        m.content("The chess move format is 'Source Square, Destination Square, (Promo Piece)'.\n\ne.g. Moving a Queen from A1 to B8 will stringify to `a1b8`.\n\nIf there is a pawn promotion involved, the piece promoted to will be appended to the end of the string, alike `a7a8q` in the case of a queen promotion.\n\nCapital Letters represent white pieces, while lower case represents black pieces.\n\nFor more help click here: https://github.com/idanmuze/threadrook/blob/master/move_guide.md").ephemeral(true)
    }).await?;

    Ok(())
}
