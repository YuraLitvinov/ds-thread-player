use crate::ds_functions::parse_input::split_message;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use reqwest::Client;
use serenity::all::*;
use songbird::Call;
use songbird::Songbird;
use songbird::input::Compose;
use songbird::input::Input;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use url;
use url::Host;
use url::Origin;
use url::Url;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};
use tracing::{event, Level};

#[allow(unused)]
#[derive(Debug)]
pub enum VoiceState {
    Free,
    Occupied,
}
pub async fn start_voice_session<'a>(
    guild: &Guild,
    msg: &'a Message,
    manager: Arc<Songbird>,
    ctx: &Context,
) {
    let channel_id = determine_if_is_voice(guild, msg, ctx).await;
    event!(Level::INFO, "{channel_id:#?}");
    let playlist = seek_youtube_playback_playlist(split_message(msg).get(1).cloned()).await;
    event!(Level::INFO, "{playlist:#?}");

    if let Some(vals) = playlist
        && let Some(val) = channel_id
    {
        let call = manager
            .join(guild.id, val)
            .await
            .expect("Couldn't make manager");
        let mut callman = call.lock().await;
        callman.set_bitrate(songbird::driver::Bitrate::BitsPerSecond(96_000));
        //Saving state of current user
        playvoice(callman, msg, manager, vals).await;
    }
}

pub async fn playvoice(
    mut callman: MutexGuard<'_, Call>,
    msg: &Message,
    manager: Arc<songbird::Songbird>,
    input: Vec<String>,
) {
    let length = input.len();
    for (index, url) in input.iter().enumerate() {
        let mut input = seek_youtube_playback(url.to_string()).await;
        let track_info = input.aux_metadata().await.unwrap().duration.unwrap();
        event!(Level::INFO, "{track_info:#?}");
        callman.play_input(input).play().expect("Couldn't play");
        event!(Level::INFO, "Emitting audiostream");
        let guild_id = msg.clone().guild_id.unwrap();
        let manager_clone = Arc::clone(&manager);
        if length >= index {
            tokio::spawn(async move {
                tokio::time::sleep(track_info).await;
                manager_clone.leave(guild_id).await.expect("Couldn't leave");
            });
        }
    }
}

#[allow(unused)]
fn split_url(url: &str) -> Result<String, Box<dyn snafu::Error>> {
    let host_youtube = Host::Domain("youtube.com".to_string());
    let u = Url::parse(url)?;
    match u.origin() {
        Origin::Tuple(_, h, _) => {
            if h == host_youtube {
                return Ok(url.to_string());
            } else {
                Ok("err".to_string())
            }
        }
        _ => Ok("err".to_string()),
    }
}

pub async fn determine_if_is_voice(
    guild: &Guild,
    msg: &Message,
    ctx: &Context,
) -> Option<ChannelId> {
    if guild.voice_states.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Голосовые каналы пусты")
            .await
            .expect("Couldn't send a message");
    }
    guild
        .voice_states
        .par_iter()
        .find_map_any(|(user_id, state)| {
            if user_id == &msg.author.id {
                state.channel_id
            } else {
                None
            }
        })
}
pub async fn next_voice_session(guild_id: GuildId, id: ChannelId, manager: Arc<Songbird>) {
    let call = manager
        .join(guild_id, id)
        .await
        .expect("Couldn't make manager");
    call.lock().await.queue().skip().expect("Couldn't skip");
}

pub async fn end_voice_session(
    guild_id: GuildId,
    manager: &Arc<Songbird>,
) -> Result<(), songbird::error::JoinError> {
    manager.remove(guild_id).await
}

pub async fn seek_youtube_playback(param: String) -> Input {
    let client = Client::new();
    let mut new_songbird = songbird::input::YoutubeDl::new(client, param);

    let par = new_songbird
        .create_async()
        .await
        .expect("Couldn't create audio stream. This is unexpected behavior");
    let input = songbird::input::Input::Live(
        songbird::input::LiveInput::Raw(par),
        Some(Box::new(new_songbird)),
    );
    input
}

pub async fn seek_youtube_playback_playlist(url: Option<String>) -> Option<Vec<String>> {
    if let Some(param) = url {
        let mut dl_new = YoutubeDl::new(&param);
        //By manipulating the playlist_items method we can increase or decrease the number of videos to be played
        let output = dl_new
            .playlist_items(2)
            .run()
            .expect("Is caused by yt-dlp not being installed");
        println!("{:#?}", output.clone().into_playlist());
        match output {
            YoutubeDlOutput::Playlist(val) => {
                if let Some(videos) = val.entries {
                    let urls = videos
                        .par_iter()
                        .filter_map(|video| {
                            if let Some(url) = &video.url {
                                Some(url.to_string())
                            } else {
                                Some(param.clone())
                            }
                        })
                        .collect::<Vec<String>>();
                    return Some(urls);
                } else {
                    return Some(vec![param]);
                }
            }
            _ => Some(vec![param]),
        }
    } else {
        return None;
    }
}

// pub async fn start_voice_session(client: &tokio_postgres::Client, guild: &Guild, msg: &Message, manager: Arc<Songbird>, ctx: Context) {
//     let channel_id = determine_if_is_voice(guild, msg, &ctx).await;
//     println!("{:#?}", channel_id);
//     let vals = seek_youtube_playback_playlist(split_message(msg).get(1).cloned());
//     println!("{:#?}", vals);
//     if let Some(vals) = vals &&
//         let Some(val) = channel_id {
//             let call = manager
//                 .join(guild.id, val)
//                 .await.expect("Couldn't make manager");
//                 let mut callman = call.lock().await;
//                 callman.set_bitrate(songbird::driver::Bitrate::Max);
//                 //Saving state of current user
//                 db::voice_status_play(client, msg, VoiceState::Occupied).await;
//                 for each in &vals {
//                     let playback = seek_youtube_playback(each.to_string()).await;
//                     callman.enqueue_input(playback).await;
//                 }

//                 let track = callman
//                     .queue()
//                     .current_queue()
//                     .iter()
//                     .next();
//                 if let Some(track) = track {
//                     let info = track.play();
//                     let k =  tokio::spawn(async move {
//                         tokio::time::sleep(info);
//                         end_voice_session(guild.id, &manager)
//                             .await
//                             .expect("Couldn't end voice session")
//                     });

//                 }

//     }
// }
