use std::{borrow::Cow, time::Duration};

use anyhow::anyhow;
use pleco::Player;
use poise::serenity_prelude::{CacheHttp, ChannelType::PublicThread, CreateThread, Member};
use tokio::{sync::mpsc, time::timeout};

use crate::{
    chess_match::{
        get_opposite_colour, ChessMatch, ChessPlayer, GameState, PlayerSlot, PlayerTime,
    },
    Context, DiscordCommand, DiscordCommunication, Error,
};

/// Create a chess match in a public thread. Opponents can join using /join_match.
#[poise::command(
    slash_command,
    required_permissions = "USE_SLASH_COMMANDS",
    required_bot_permissions = "VIEW_CHANNEL | SEND_MESSAGES | MANAGE_MESSAGES | EMBED_LINKS | READ_MESSAGE_HISTORY | USE_SLASH_COMMANDS | MANAGE_THREADS | CREATE_PUBLIC_THREADS | CREATE_PRIVATE_THREADS | SEND_MESSAGES_IN_THREADS",
    user_cooldown = "30"
)]
pub async fn create_match(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let creating_member: Cow<'_, Member>;

    if let Some(member) = ctx.author_member().await {
        creating_member = member
    } else {
        return Err(anyhow!("Unable to get Player_1 Member").into());
    }

    // Check if member is already in a match within this guild.
    let (response_channel_tx, mut response_channel_rx) = mpsc::channel(1);
    ctx.data()
        .system_communication_channel
        .0
        .send(DiscordCommunication(
            creating_member.guild_id,
            DiscordCommand::VerifyIfAlreadyInMatch(
                Box::new(creating_member.clone().into_owned()),
                response_channel_tx,
            ),
        ))?;
    match timeout(Duration::from_secs(10), response_channel_rx.recv()).await {
        Ok(_) => {
            ctx.say("You are already in a match. Threadrook currently only supports users competing in a single match at a time per server.").await?;
            return Ok(());
        }
        Err(_) => {
            ctx.say("Creating match...").await?;
        }
    }

    let match_thread_message = ctx
        .say(format!(
            "{} just created a chess match! Use `/join_match` to join.",
            ctx.author().name,
        ))
        .await?;

    let match_thread_message_clone = match_thread_message.clone();

    let match_thread = ctx
        .channel_id()
        .create_public_thread(
            ctx.http(),
            match_thread_message.into_message().await?,
            |t| -> &mut CreateThread {
                t.name(format!("{}'s ThreadRook Chess Match", ctx.author().name,))
                    .kind(PublicThread)
            },
        )
        .await?;

    match_thread
        .say(
            ctx.http(),
            format!(
                "
        <@{}>
        \nWelcome!
        \n`/create_match` to create your own match. 
        \n`/join_match` to join the match of any user that is looking for an opponent. 
        \n`/make_move` to make a chess move.
        \n`/move_notation_guide` for a quick guide on Threadrook chess move notation. 
        \n`/resign` to forfeit. 
        \nLearn more about ThreadRook at https://github.com/idanmuze/threadrook",
                ctx.author().id
            ),
        )
        .await?;

    let player_1_colour = if rand::random() {
        Player::White
    } else {
        Player::Black
    };

    let player_2_colour = get_opposite_colour(player_1_colour);

    let chess_match = ChessMatch::builder()
        .state(GameState::WaitingForOpponent)
        .opponent_join_deadline(90)
        .player_one(
            ChessPlayer::builder()
                .user_id(creating_member.user.id)
                .player_slot(PlayerSlot::Player1)
                .in_game_representation(player_1_colour)
                .build(),
        )
        .player_two(
            ChessPlayer::builder()
                .player_slot(PlayerSlot::Player2)
                .in_game_representation(player_2_colour)
                .build(),
        )
        .player_time(PlayerTime::builder().white(300).black(300).build())
        .build();

    chess_match
        .spawn(
            ctx,
            match_thread_message_clone.into_message().await?,
            match_thread,
            Box::new(creating_member.clone().into_owned()),
        )
        .await?;

    Ok(())
}
