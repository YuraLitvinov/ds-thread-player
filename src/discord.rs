use crate::BROKER;
use crate::db::run_query;
use crate::ds_functions::function_voice::VoiceState;
use crate::ds_functions::function_voice::{
    determine_if_is_voice, end_voice_session, next_voice_session, start_voice_session,
};
use crate::ds_functions::parse_input::split_message;
use serenity::all::Guild;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::{Songbird, register_with};
use tracing::{Level, event};
pub struct Handler;

pub struct CtxMsg {
    pub ctx: serenity::client::Context,
    pub msg: Message,
}
#[derive(Debug)]
pub enum CommandType {
    Play,
    Leave,
    Skip,
    Help,
    NotCommand,
}
pub trait Command {
    fn command_type(&self) -> CommandType;
}

impl Command for str {
    fn command_type(&self) -> CommandType {
        match self {
            "!play" => CommandType::Play,
            "!leave" => CommandType::Leave,
            "!skip" => CommandType::Skip,
            "!help" => CommandType::Help,
            _ => CommandType::NotCommand,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, _ctx: Context, guild: Guild, is_new: Option<bool>) {
        event!(Level::INFO, "Availability status: {}  (guild: {:#?})", guild.name, is_new)
    }

    async fn message(&self, ctx: Context, msg: Message) {
        BROKER
            .0
            .send(std::sync::Arc::new(CtxMsg { ctx, msg }))
            .await
            .expect("BROKER SEND ERROR");
    }
}

pub async fn start_client() {
    // Login with a bot token from the environment
    let token = std::env::var("DISCORD_API_KEY").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let client = Client::builder(&token, intents).event_handler(Handler);
    event!(Level::INFO, "{:#?}", client.get_intents());
    let songbird = Songbird::serenity();

    register_with(client, songbird.clone())
        .voice_manager_arc(songbird)
        .await
        .expect("Error while registering the voice client")
        .start()
        .await
        .expect("Couldn't start client");
}

pub async fn read_msg(
    client: &std::sync::Arc<Option<tokio_postgres::Client>>,
) -> Result<(), Box<dyn snafu::Error>> {
    let mut rx = BROKER.1.lock().await;
    let broker = rx.recv().await.ok_or("Couldn't receive response")?;

    let manager = songbird::get(&broker.ctx)
        .await
        .expect("Couldn't init manager");

    let guild = broker
        .msg
        .guild_id
        .ok_or("Guild ID not found")?
        .to_guild_cached(&broker.ctx.cache)
        .ok_or("Guild not found")?
        .clone();

    if  let Some(arg) = split_message(&broker.msg).first()
    {
        let args = arg.as_str().command_type();
        if let Some(some_client) = &**client {
            run_query(&some_client, &broker.msg, &args).await;
        }

        match args {
            CommandType::Play => {
                if let Some(some_client) = &**client {
                    crate::db::voice_status_play(some_client, &broker.msg, VoiceState::Occupied).await;
                }
                tokio::spawn(async move {
                    start_voice_session(&guild, &broker.msg, manager, &broker.ctx).await;
                });
            }
            CommandType::Help => {
                broker
                    .msg
                    .channel_id
                    .say(
                        &broker.ctx.http,
                        "!play https://someurl.com - Starts the bot replacing someurl with your link
                        !leave - Bot leaves the voice channel",
                    )
                    .await
                    .expect("Couldn't send a message");
            }
            CommandType::Leave => {
                //Manage user permissions or inhibit user who initialized the command
                if let Err(e) = end_voice_session(guild.id, &manager).await {
                    event!(Level::ERROR, "{e}")
                }
            }
            CommandType::Skip => {
                if let Some(val) = determine_if_is_voice(&guild, &broker.msg, &broker.ctx).await {
                    next_voice_session(guild.id, val, manager).await;
                }
            }

            CommandType::NotCommand => event!(Level::INFO, "Type is not command"),
        }
    };
    Ok(())
}
