
#!/bin/bash
#May be useful for running locally and debugging depending on the chosen alias
mkdir build 
set -euo pipefail
docker build -t 'thread-player-win' -f Dockerfile.win .
container_id=$(docker create 'thread-player-win')
docker cp $container_id:/app/target/x86_64-pc-windows-gnu/release/ds_thread_player.exe ./build/ds_thread_player-win-x86_64.exe
