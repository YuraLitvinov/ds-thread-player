## A fairly simple solution to add a youtube music player bot to your server.
### There are a few dependencies, that have to be dealt with first on Windows-based systems.
### You can choose to install postgresql, which I am planning on extending to trace user privileges to avoid taking control over bot. 
    The crucial dependency, that has to be installed at all costs is yt-dlp. Windows users can do this by installing it winget repo with: winget install --id=yt-dlp.yt-dlp  -e
### Linux users can enjoy a higher level of simplicity, as they can install it from dockerfile and run it in a container using the run_example.sh file and editing the contents of the file to include discord api key. 
    The key be accessed from the discord developers portal: https://discord.com/developers/applications
    Then, create a new application, set your name of choice. 
    have to install it manually by obtaining the install link for `guild install` from installation section.
    Go to `bot` section, click `reset token` and you will have your key, which you then have to save to be able to use in the future, for Docker: using run_example.sh which you can build with build.sh and then freely run. 
    For Windows: rename .env.example -> .env and insert your key where it says DISCORD_API_KEY.