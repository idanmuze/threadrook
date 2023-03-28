use std::time::Duration;

use anyhow::anyhow;
use pleco::{Board, Player};
use poise::serenity_prelude::{CacheHttp, GuildChannel, Member, Message, UserId};
use tokio::{task::JoinHandle, time::interval};

use crate::{Context, DiscordCommand, DiscordCommunication, Error};

#[derive(Debug, Clone, Copy, buildstructor::Builder)]
pub struct ChessMatch {
    state: GameState,
    opponent_join_deadline: i32,
    player_one: ChessPlayer,
    player_two: ChessPlayer,
    player_time: PlayerTime,
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    WaitingForOpponent,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, buildstructor::Builder)]
pub struct ChessPlayer {
    pub user_id: Option<UserId>,
    player_slot: PlayerSlot,
    in_game_representation: Player,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerSlot {
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy, buildstructor::Builder)]
pub struct PlayerTime {
    white: i32,
    black: i32,
}

#[derive(Debug, Clone, buildstructor::Builder)]
struct MatchFrontend {
    match_thread_message: Message,
    match_thread: GuildChannel,
    board_message: Message,
    time_info_message: Message,
    legal_move_message: Message,
}

impl ChessMatch {
    pub async fn spawn(
        mut self,
        ctx: Context<'_>,
        match_thread_message: Message,
        match_thread: GuildChannel,
        player_1_member: Box<Member>,
    ) -> Result<(), Error> {
        let board = Board::start_pos();

        let board_message = match_thread.say(ctx.http(), board.pretty_string()).await?;

        let time_info_message = match_thread
            .say(
                ctx.http(),
                format!(
                    "Deadline for an opponent to join: {}",
                    self.opponent_join_deadline
                ),
            )
            .await?;

        let legal_move_message = match_thread
            .say(
                ctx.http(),
                "White's legal moves in the current position: _".to_string(),
            )
            .await?;

        legal_move_message.pin(ctx.http()).await?;
        time_info_message.pin(ctx.http()).await?;
        board_message.pin(ctx.http()).await?;

        let mut frontend = MatchFrontend::builder()
            .match_thread_message(match_thread_message)
            .match_thread(match_thread)
            .board_message(board_message)
            .time_info_message(time_info_message)
            .legal_move_message(legal_move_message)
            .build();

        let guild_id = player_1_member.guild_id;
        let time_ticker_tx = ctx.data().system_communication_channel.0.clone();
        let mut time_ticker_interval = interval(Duration::from_secs(1));
        let time_ticker_task = tokio::spawn(async move {
            loop {
                if time_ticker_tx
                    .send(DiscordCommunication(guild_id, DiscordCommand::TimeTick))
                    .is_ok()
                {
                    time_ticker_interval.tick().await;
                }
            }
        });

        let mut system_communication_rx = ctx.data().system_communication_channel.0.subscribe();
        while let Ok(communication) = system_communication_rx.recv().await {
            if communication.0 == player_1_member.guild_id {
                match communication.1 {
                    DiscordCommand::JoinMatch(waiting_member, joining_member) => {
                        if player_1_member.user.id == waiting_member.user.id {
                            frontend
                                .match_thread
                                .say(
                                    ctx.http(),
                                    format!("{} just joined", joining_member.user.name),
                                )
                                .await?;

                            self.player_two.user_id = Some(joining_member.user.id);
                            self.player_turns(
                                time_ticker_task,
                                ctx,
                                board,
                                player_1_member,
                                joining_member,
                                frontend,
                            )
                            .await?;

                            break;
                        }
                    }
                    DiscordCommand::VerifyIfAlreadyInMatch(member, respond_tx) => {
                        if member.user.id == player_1_member.user.id {
                            respond_tx.send(true).await?;
                        }
                    }
                    DiscordCommand::TimeTick => {
                        self.opponent_join_deadline -= 1;
                        frontend
                            .time_info_message
                            .edit(ctx.http(), |m| {
                                m.content(format!(
                                    "Deadline for an opponent to join: {}",
                                    self.opponent_join_deadline
                                ))
                            })
                            .await?;

                        if self.opponent_join_deadline == 0 {
                            time_ticker_task.abort();
                            self.end_the_game(ctx, frontend).await?;

                            break;
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }

    async fn player_turns(
        mut self,
        time_ticker_task: JoinHandle<()>,
        ctx: Context<'_>,
        mut board: Board,
        player_1_member: Box<Member>,
        player_2_member: Box<Member>,
        mut frontend: MatchFrontend,
    ) -> Result<(), Error> {
        self.state = GameState::Playing;
        let mut acting_player_colour = Player::White;

        frontend
            .match_thread
            .say(ctx.http(), "The match has now started!")
            .await?;

        frontend
            .time_info_message
            .edit(ctx.http(), |m| {
                m.content(format!(
                    "{} ({}) Time: {}\n{} ({}) Time: {}",
                    player_1_member.user.name,
                    self.player_one.in_game_representation,
                    self.get_colour_timeleft(self.player_one.in_game_representation),
                    player_2_member.user.name,
                    self.player_two.in_game_representation,
                    self.get_colour_timeleft(self.player_two.in_game_representation),
                ))
            })
            .await?;

        frontend
            .legal_move_message
            .edit(ctx.http(), |m| {
                m.content(format!(
                    "{}'s legal moves in the current position:\n{:?}",
                    acting_player_colour,
                    board
                        .generate_moves()
                        .iter_mut()
                        .map(|chess_move| chess_move.stringify())
                        .collect::<Vec<String>>()
                ))
            })
            .await?;

        let mut system_communication_rx = ctx.data().system_communication_channel.0.subscribe();
        while let Ok(communication) = system_communication_rx.recv().await {
            if communication.0 == player_1_member.guild_id {
                match communication.1 {
                    DiscordCommand::JoinMatch(_, _) => todo!(),
                    DiscordCommand::MakeMove(query_member, chess_move) => {
                        if let Ok(query_player) = self.check_if_member_is_in_game(
                            query_member,
                            &player_1_member,
                            &player_2_member,
                        ) {
                            if query_player
                                == self.get_acting_chessplayer(acting_player_colour).unwrap()
                            {
                                let legal_moves = board.generate_moves();

                                let stringified_legal_moves = board
                                    .generate_moves()
                                    .iter_mut()
                                    .map(|chess_move| chess_move.stringify())
                                    .collect::<Vec<String>>();

                                if stringified_legal_moves.contains(&chess_move) {
                                    if let Some(bit_move) = legal_moves.get(
                                        stringified_legal_moves
                                            .iter()
                                            .position(|m| m == &chess_move)
                                            .unwrap(),
                                    ) {
                                        board.apply_move(*bit_move);

                                        frontend
                                            .board_message
                                            .edit(ctx.http(), |m| m.content(board.pretty_string()))
                                            .await?;

                                        frontend
                                            .match_thread
                                            .say(
                                                ctx.http(),
                                                format!(
                                                    "{} ({}) made the move {}.",
                                                    get_member_from_chessplayer(
                                                        query_player,
                                                        &player_1_member,
                                                        &player_2_member
                                                    )
                                                    .unwrap(),
                                                    query_player.in_game_representation,
                                                    chess_move
                                                ),
                                            )
                                            .await?;

                                        // Check for a checkmate.
                                        if board.checkmate() {
                                            frontend
                                                .match_thread
                                                .say(
                                                    ctx.http(),
                                                    format!(
                                                        "{} checkmated {}. GG.",
                                                        player_1_member, player_2_member
                                                    ),
                                                )
                                                .await?;
                                            break;
                                        }

                                        // Check for a stalemate.
                                        if board.stalemate() {
                                            frontend
                                                .match_thread
                                                .say(
                                                    ctx.http(),
                                                    format!(
                                                        "{} caused a stalemate {}. GG.",
                                                        player_1_member, player_2_member
                                                    ),
                                                )
                                                .await?;
                                            break;
                                        }

                                        acting_player_colour = get_opposite_colour(
                                            query_player.in_game_representation,
                                        );

                                        frontend
                                            .legal_move_message
                                            .edit(ctx.http(), |m| {
                                                m.content(format!(
                                                    "{}'s legal moves in the current position:\n{:?}",
                                                    acting_player_colour,
                                                    board
                                                        .generate_moves()
                                                        .iter_mut()
                                                        .map(|chess_move| chess_move.stringify())
                                                        .collect::<Vec<String>>()
                                                ))
                                            })
                                            .await?;
                                    } else {
                                        return Err(anyhow!("stringified_legal_moves index is out of range of legal_moves. Should be unreacheable").into());
                                    }
                                } else {
                                    frontend
                                        .match_thread
                                        .say(
                                            ctx.http(),
                                            format!(
                                        "{} is not a legal move. Use `/move_notation_guide` for help.",
                                        chess_move
                                    ),
                                        )
                                        .await?;
                                }
                            } // else { It is not query_member's turn. }
                        } else {
                            frontend
                                .match_thread
                                .say(ctx.http(), "You are not a player in this match.")
                                .await?;
                        }
                    }
                    DiscordCommand::Resign(_) => todo!(),
                    DiscordCommand::VerifyIfAlreadyInMatch(member, respond_tx) => {
                        if self
                            .check_if_member_is_in_game(member, &player_1_member, &player_2_member)
                            .is_ok()
                        {
                            respond_tx.send(true).await?;
                        }
                    }
                    DiscordCommand::TimeTick => {
                        match self
                            .get_acting_chessplayer(acting_player_colour)
                            .unwrap()
                            .in_game_representation
                        {
                            Player::White => {
                                self.player_time.white -= 1;
                            }
                            Player::Black => {
                                self.player_time.black -= 1;
                            }
                        }

                        frontend
                            .time_info_message
                            .edit(ctx.http(), |m| {
                                m.content(format!(
                                    "{} ({}) Time: {}\n{} ({}) Time: {}",
                                    player_1_member.user.name,
                                    self.player_one.in_game_representation,
                                    self.get_colour_timeleft(
                                        self.player_one.in_game_representation
                                    ),
                                    player_2_member.user.name,
                                    self.player_two.in_game_representation,
                                    self.get_colour_timeleft(
                                        self.player_two.in_game_representation
                                    ),
                                ))
                            })
                            .await?;

                        if self.get_colour_timeleft(acting_player_colour) == 0 {
                            frontend
                                .match_thread
                                .say(
                                    ctx.http(),
                                    format!(
                                        "{} just lost on time. {} wins. GG.",
                                        acting_player_colour,
                                        get_opposite_colour(acting_player_colour)
                                    ),
                                )
                                .await?;

                            time_ticker_task.abort();

                            break;
                        }
                    }
                }
            }
        }

        self.end_the_game(ctx, frontend).await?;
        Ok(())
    }

    async fn end_the_game(
        mut self,
        ctx: Context<'_>,
        frontend: MatchFrontend,
    ) -> Result<(), Error> {
        self.state = GameState::GameOver;

        frontend
            .match_thread
            .say(
                ctx.http(),
                "The match is over. Deleting thread in 30 secs...",
            )
            .await?;

        tokio::time::sleep(Duration::from_secs(30)).await;

        frontend.match_thread.delete(ctx.http()).await?;
        frontend.match_thread_message.delete(ctx.http()).await?;

        Ok(())
    }

    fn get_acting_chessplayer(self, acting_player_colour: Player) -> Result<ChessPlayer, Error> {
        if self.player_one.in_game_representation == acting_player_colour {
            Ok(self.player_one)
        } else if self.player_two.in_game_representation == acting_player_colour {
            Ok(self.player_two)
        } else {
            Err(anyhow!(
                "Neither of the players matched the turn colour. Should be impossible to reach."
            )
            .into())
        }
    }

    fn check_if_member_is_in_game(
        self,
        query_member: Box<Member>,
        player_1_member: &Member,
        player_2_member: &Member,
    ) -> Result<ChessPlayer, Error> {
        if query_member.user.id == player_1_member.user.id {
            Ok(self.player_one)
        } else if query_member.user.id == player_2_member.user.id {
            Ok(self.player_two)
        } else {
            Err(anyhow!("This member is not playing in this match.").into())
        }
    }

    fn get_colour_timeleft(self, query_colour: Player) -> i32 {
        match query_colour {
            Player::White => self.player_time.white,
            Player::Black => self.player_time.black,
        }
    }
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
