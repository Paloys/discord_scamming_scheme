# A local FTP server that stores files on discord

## How to build
```
cargo build
```

## How to run
First you need to setup some environement variables: `DISCORD_TOKEN` for the token the bot will be using and `DISCORD_CHANNEL_ID` for the id for the channel the bot will store the files in. Then you can directly run it:
```
cargo run
```
and access the FTP server on localhost:2121.
