use anyhow::Context;
use poise::serenity_prelude::{self as serenity};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use tokio::sync::broadcast;

use threadrook::{
    commands::{create_match::*, join_match::*, make_move::*, move_notation_guide::*, resign::*},
    Data, Error,
};

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    // broadcast channel that allows both external Discord command invocations and internal, spawned tasks to communicate with chess matches

    // Chess matches actually only use one broadcast receiver at a time.
    // The reason why an mpsc wasn't used was because there is no way of utilizing a receiver without it being mutable. Data references are immutable.
    let (tx, rx) = broadcast::channel(10000);

    // Get the discord token set in `Secrets.toml`.
    // Make sure to create one before running.
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                create_match(),
                join_match(),
                make_move(),
                move_notation_guide(),
                resign(),
            ],
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    system_communication_channel: (tx, rx),
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}
