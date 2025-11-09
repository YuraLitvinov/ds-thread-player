use crate::{
    discord::CommandType, ds_functions::function_voice::VoiceState, error::snafu_error::StdEnvSnafu,
};
use serenity::all::Message;
use snafu::ResultExt;
use tokio_postgres::{Client, NoTls};
use tracing::{Level, event};

pub async fn init_db() -> Result<Client, Box<dyn snafu::Error>> {
    let (uservar, hostvar, passvar) = ("POSTGRES_USER", "POSTGRES_HOST", "POSTGRES_PASSWORD");

    let user = std::env::var(uservar).context(StdEnvSnafu { varname: uservar })?;
    let password = std::env::var(passvar).context(StdEnvSnafu { varname: passvar })?;
    let host = std::env::var(hostvar).context(StdEnvSnafu { varname: hostvar })?;

    let connection_data = format!("host={host} user={user} password={password}");

    let (client, connection_socket) = 
        tokio_postgres::connect(
            &connection_data, 
            NoTls
        ).await?;

    tokio::spawn(async move { 
        if let Err(e) = connection_socket.await {
            event!(Level::ERROR, "{e:#?}");
        }
    });

    event!(Level::INFO, "{client:#?}");
    Ok(client)
}

pub async fn _run_voice_conference(
    client: &Client,
    msg: &Message,
) -> Result<(), Box<dyn snafu::Error>> {
    let _author_id = msg.author.id.get() as usize;
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let query_over_messages = client
        .query(
            &format!(
                "SELECT * FROM guild_{}.voice_state
                ORDER BY timestamp DESC",
                guild_id
            ),
            &[],
        )
        .await?;
    let _state = query_over_messages
        .into_iter()
        .map(|each| {
            let state = match each.get::<&str, &str>("state") {
                "O" => VoiceState::Occupied,
                _ => VoiceState::Free,
            };
            state
        })
        .collect::<Vec<VoiceState>>()
        .first();

    Ok(())
}

pub async fn voice_status_play<'a>(client: &'a Client, msg: &'a Message, state: VoiceState) {
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let message_id = msg.id.get() as i64;
    let channel_id = msg.channel_id.get() as i64;
    let author_id = msg.author.id.get() as i64;
    let timestamp = msg
        .timestamp
        .to_string()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .expect("Couldn't parse");
    /*let _request = {
        std::sync::Arc::new(Request {
            request_type: crate::discord::CommandType::NotCommand,
            guild_id: msg.guild_id.unwrap(),
            msg_id: msg.id,
            channel_id: msg.channel_id,
            author_id: msg.author.id,
            timestamp,
            state: VoiceState::Occupied,
        })
    };
    */

    let state = match state {
        VoiceState::Free => "F",
        VoiceState::Occupied => "O",
    };

    let schema_name = format!("CREATE SCHEMA IF NOT EXISTS guild_{}", guild_id);
    if let Err(e) = client.execute(&schema_name, &[]).await {
        event!(Level::ERROR, "{e:#?}");
    }

    let table_name = format!(
        "CREATE TABLE IF NOT EXISTS guild_{}.voice_state (
        id              SERIAL PRIMARY KEY,
        message_id       BIGINT NOT NULL,
        channel_id      BIGINT NOT NULL,
        author_id       BIGINT NOT NULL,
        state           VARCHAR NOT NULL,
        timestamp       TIMESTAMPTZ NOT NULL
    )",
        guild_id
    );
    if let Err(e) = client.execute(&table_name, &[]).await {
        event!(Level::ERROR, "{e}")
    }

    if let Err(e) = client
        .execute(
            &format!(
                "
            INSERT INTO guild_{}.voice_state (message_id, channel_id, author_id, state, timestamp)
            VALUES ($1, $2, $3, $4, $5)
            ",
                guild_id
            ),
            &[&message_id, &channel_id, &author_id, &state, &timestamp],
        )
        .await {
            event!(Level::ERROR, "{e}")
        }
}

pub async fn run_query(client: &Client, msg: &Message, message_type: &CommandType) {
    let guild_id = msg.guild_id.unwrap().get() as i64;
    let message_id = msg.id.get() as i64;
    let channel_id = msg.channel_id.get() as i64;
    let author_id = msg.author.id.get() as i64;
    let author_insignia = &msg.author.name;
    let contents = &msg.content;
    let timestamp = msg
        .timestamp
        .to_string()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .expect("Couldn't parse");
    let message_type = format!("{:#?}", message_type);

    let schema_name = format!("CREATE SCHEMA IF NOT EXISTS guild_{}", guild_id);
    if let Err(e) = client.execute(&schema_name, &[]).await {
        event!(Level::ERROR, "{e}")
    }

    let table_name = format!(
        "CREATE TABLE IF NOT EXISTS guild_{}.typed_messages (
        message_id       BIGINT PRIMARY KEY,
        channel_id      BIGINT NOT NULL,
        author_id       BIGINT NOT NULL,
        author_insignia TEXT,
        contents         TEXT,
        message_type    TEXT,
        timestamp       TIMESTAMPTZ NOT NULL
    )",
        guild_id
    );

    if let Err(e) = client.execute(&table_name, &[]).await {
        event!(Level::ERROR, "{e}")
    }

    if let Err(e) = client
        .execute(
            &format!("
            INSERT INTO guild_{}.typed_messages (message_id, channel_id, author_id, author_insignia, contents, message_type, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (message_id) DO NOTHING
            ", guild_id),
            &[
                &message_id,
                &channel_id,
                &author_id,
                &author_insignia,
                &contents,
                &message_type,
                &timestamp
            ],
        )
        .await {
            event!(Level::ERROR, "{e}")
        }
}
