
#!/bin/bash
#May be useful for running locally and debugging depending on the chosen alias
mkdir build 

set -euo pipefail
docker build -t 'thread-player' -f Dockerfile.build .
container_id=$(docker create 'thread-player')
docker cp $container_id:/app/target/release/ds_thread_player ./build/ds_thread_player-linux-x86_64
