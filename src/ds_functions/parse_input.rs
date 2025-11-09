use rayon::prelude::*;
use serenity::all::Message;
use tracing::{Level, event};
pub fn split_message(msg: &Message) -> Vec<String> {
    let args = msg
        .content
        .split_ascii_whitespace()
        .collect::<Vec<&str>>()
        .par_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    event!(Level::INFO, "{args:#?}");
    args
}
