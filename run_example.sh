docker run -m 512mb \
 -e POSTGRES_USER="postgres" \
 -e POSTGRES_PASSWORD="postgres" \
 -e POSTGRES_HOST="localhost" \
 -e DISCORD_API_KEY="<your-key-here>" \
 -v $(pwd)/build/discord_linux_thread_player_merger:/usr/local/bin/discord_linux_thread_player_merger thread-player