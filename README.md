# tagbrain

Self-hosted app to automatically tag your music. It uses AcoustID to identify
music and musicbrainz to get tags.

This app is still in development and future update may break config or database
without migration.

## Installation & Usage

I recommend to use docker-compose to run this app.

1. Create `compose.yaml` file like this:

```yaml
version: "3"
services:
  app:
    image: nazo6/tagbrain
    ports:
      - 3090:3080
    volumes:
      - ./config:/config # folder to save config
      - ./data:/data # folder to save db
      - ./source:/source # folder with music to tag
      - ./target:/target # folder to save tagged music
    environment:
      - ACOUST_ID_API_KEY=your_api_key
```

or you can use `compose.yaml` in this repo if you want you build from source.

`ACOUST_ID_API_KEY` is required to use AcoustID API. You can get it from
[https://acoustid.org/](https://acoustid.org/). After config file is created,
environment variable will not be used anymore.

2. Run `docker compose up -d` to start app.
3. Open `http://<ip>:3090` and you will see the UI. Of course, you can change
   port in `compose.yaml`.
4. Scan is executed when you copy music files to `source` folder. Alternatively,
   you can start scan manually from UI.
5. You can see execution log from UI. But you can find more detailed log in
   container log.

## Config

After first launch, a config file will be created in `config` folder. Plaease
see created `config.toml` for more info. You have to restart the app after
config changes.
