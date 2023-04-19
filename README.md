# AI-Waifu-Vtuber on rust
[Reference project](https://github.com/ardha27/AI-Waifu-Vtuber)

## Required services
- https://hub.docker.com/r/missuo/deeplx
- https://github.com/ololoshka2871/Selerio-TTS-server
- https://github.com/ololoshka2871/Voice-2-txt-UrukHan

### Constants
* Discord Voice output 2ch @ 48000 Hz

## How to use
1. Run docker container witn deeplx bridge
2. Run Selerio-TTS-server
3. Run Voice-2-txt-UrukHan
4. Copy `config.json.init` to `config.json` and fill it
5. Run `cargo run --release --bin ai-waifu-interactive` or `cargo run --release --bin ai-waifu-discord-bot`