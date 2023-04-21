# AI-Waifu-Vtuber on rust
[Reference project](https://github.com/ardha27/AI-Waifu-Vtuber)

## References
- https://hub.docker.com/r/missuo/deeplx -> integrated

## Required services
- https://github.com/ololoshka2871/Selerio-TTS-server
- https://github.com/ololoshka2871/Voice-2-txt-UrukHan

### Constants
* Discord Voice output 2ch @ 48000 Hz

## How to use
1. Create python venv: `python3 -m venv venv`
1. Activate venv: `source venv/bin/activate`
1. Install requirements: `pip install -r requirements.txt`
1. Run Selerio-TTS-server
1. Run Voice-2-txt-UrukHan
1. Copy `config.json.init` to `config.json` and fill it
1. Run `cargo run --release --bin ai-waifu-interactive` or `cargo run --release --bin ai-waifu-discord-bot`