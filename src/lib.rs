pub mod chess_match;
pub mod commands;

use poise::serenity_prelude::{GuildId, Member};
use tokio::sync::{broadcast, mpsc};
extern crate pleco;

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
