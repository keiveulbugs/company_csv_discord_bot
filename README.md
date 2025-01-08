You can start the bot with the basic `cargo run --release` command.

Or use Docker through this:

```
sudo docker build -t message-to-csv-discordbot .

sudo docker run -dit --restart unless-stopped --name running-csvbot message-to-csv-discordbot
```

