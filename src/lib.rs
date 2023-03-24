pub mod chess_match;
pub mod commands;

use chess_match::ChessPlayer;
use tokio::sync::{broadcast, mpsc};

use anyhow::anyhow;
use poise::serenity_prelude::{GuildId, Member};

extern crate pleco;
use pleco::Player;

// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {
    pub system_communication_channel: (
        broadcast::Sender<DiscordCommunication>,
        broadcast::Receiver<DiscordCommunication>,
    ),
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Clone)]
pub struct DiscordCommunication(GuildId, DiscordCommand);

#[derive(Debug, Clone)]
enum DiscordCommand {
    JoinMatch(Box<Member>, Box<Member>),
    MakeMove(Box<Member>, String),
    Resign(Box<Member>),
    VerifyIfAlreadyInMatch(Box<Member>, mpsc::Sender<bool>),
    TimeTick,
}

pub fn get_opposite_colour(colour: Player) -> Player {
    if colour == Player::White {
        Player::Black
    } else {
        Player::White
    }
}

fn get_member_from_chessplayer(
    query_chessplayer: ChessPlayer,
    player_1_member: &Member,
    player_2_member: &Member,
) -> Result<Member, Error> {
    if query_chessplayer.user_id.unwrap() == player_1_member.user.id {
        Ok(player_1_member.clone())
    } else if query_chessplayer.user_id.unwrap() == player_2_member.user.id {
        Ok(player_2_member.clone())
    } else {
        Err(
            anyhow!("Chessplayer user id did not match either of the two members in this match.")
                .into(),
        )
    }
}
