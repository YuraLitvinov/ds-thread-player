mod tests {
    use reqwest::Client;
    use songbird::input::Compose;
    use youtube_dl::YoutubeDl;

    use crate::db::init_db; //yaml::parser::return_prompt};
    #[test]
    fn test_youtube() {
        let output = YoutubeDl::new("https://www.youtube.com/watch?v=vMCbJB4yNXo")
            .socket_timeout("15")
            .run()
            .unwrap();
        let single = output.into_single_video().unwrap();
        println!("{:#?}", single);

        assert_eq!(true, false)
    }
    #[test]
    fn youtube_songbird() {
        let client = Client::new();
        let param = "https://www.youtube.com/watch?v=vMCbJB4yNXo";
        let mut new_songbird = songbird::input::YoutubeDl::new(client, param);
        let _a = new_songbird.create().unwrap();
        println!("{:#?}", new_songbird);

        assert_eq!(true, false)
    }
    #[tokio::test]
    async fn test_read_db() {
        let client = init_db().await.unwrap();
        let query_over_messages = client
            .query(
                &format!("SELECT * FROM guild_1372577191019417784.typed_messages",),
                &[],
            )
            .await
            .unwrap();
        let state = query_over_messages;
        for column in state {
            let message_id: i64 = column.get("message_id");
            let channel_id: i64 = column.get("channel_id");
            let author_id: i64 = column.get("author_id");

            println!(
                "message_id: {} channel_id: {} author_id: {}",
                message_id, channel_id, author_id
            );
        }
        assert_eq!(true, false)
    }

}
